pub mod linux;

use crate::cpu::Registers;
use crate::decoder::decode;
use crate::emitter::Emitter;
use linux::ExecutableMemory;
use std::collections::BTreeMap;

pub struct CodeBlock {
  pub offset: usize,
  pub length: usize,
}

pub struct CodeCache {
  exec_memory: ExecutableMemory,
  code_blocks: BTreeMap<usize, CodeBlock>,
  jump_cache: BTreeMap<usize, usize>,
  write_cursor: usize,
}

impl CodeCache {
  pub fn new() -> Self {
    Self {
      exec_memory: ExecutableMemory::new(),
      code_blocks: BTreeMap::new(),
      jump_cache: BTreeMap::new(),
      write_cursor: 0,
    }
  }

  pub fn get_memory_start_address(&self) -> usize {
    self.exec_memory.get_memory_area().as_ptr() as *const () as usize
  }

  pub fn get_address_for_ip(&self, ip: usize) -> Option<usize> {
    match self.code_blocks.get(&ip) {
      Some(block) => return Some(block.offset),
      None => self.jump_cache.get(&ip).copied(),
    }
  }

  pub fn translate_code_block(&mut self, code: &Box<[u8]>, ip: usize) -> usize {
    let mut write_cursor = self.write_cursor;
    let starting_offset = write_cursor;
    println!("WRITING AT {:X}", starting_offset);

    self.exec_memory.make_writable();
    let translated = self.exec_memory.get_memory_area_mut();
    let emitter = Emitter::new();
    write_cursor += emitter.encode_prelude(&mut translated[write_cursor..]);

    let mut block_ended = false;
    let mut index = ip;
    while !block_ended && index < code.len() {
      let (next_op, length, _cycles) = decode(&code[index..]);
      index += length;
      block_ended = next_op.is_block_end();
      let written = emitter.encode_op(next_op, length, &mut translated[write_cursor..]);
      write_cursor += written;
    }

    write_cursor += emitter.encode_epilogue(&mut translated[write_cursor..]);

    self.exec_memory.make_executable();
    self.write_cursor = write_cursor;
    println!("ENDED AT {:X}", write_cursor);
    starting_offset
  }

  pub fn call(&self, offset: usize, registers: &mut Registers) {
    let func_pointer = (self.get_memory_start_address() + offset) as *const ();
    let func: extern "C" fn(*const Registers) -> usize = unsafe {
      std::mem::transmute(func_pointer)
    };
    func(registers as *const Registers);
  }
}
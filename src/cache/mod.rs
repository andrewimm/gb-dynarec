pub mod blocks;
#[cfg(unix)]
pub mod linux;
#[cfg(windows)]
pub mod windows;

use blocks::{CachedBlocks, CodeBlock};
use crate::cpu::Registers;
use crate::decoder::decode;
use crate::emitter::Emitter;
use crate::mem::MemoryAreas;

#[cfg(unix)]
use linux::ExecutableMemory;
#[cfg(windows)]
use self::windows::ExecutableMemory;

pub const INITIAL_MEMORY_SIZE: usize = 0x10000;
pub const MEMORY_MINIMUM_SIZE: usize = 0x1000;
pub const MEMORY_SIZE_INCREASE: usize = 0x1000;

pub struct CodeCache {
  exec_memory: ExecutableMemory,
  code_blocks: CachedBlocks,
  write_cursor: usize,
}

impl CodeCache {
  pub fn new() -> Self {
    Self {
      exec_memory: ExecutableMemory::new(),
      code_blocks: CachedBlocks::new(),
      write_cursor: 0,
    }
  }

  pub fn get_memory_start_address(&self) -> usize {
    self.exec_memory.get_memory_area().as_ptr() as *const () as usize
  }

  pub fn get_address_for_ip(&self, ip: usize) -> Option<usize> {
    let gb_ip = ip as u16;
    self.code_blocks
      .get_region(gb_ip)
      .and_then(|region| region.get(gb_ip))
      .and_then(|block| Some(block.offset))
  }

  pub fn get_executable_memory_segment(&self, ip: usize, mem_ptr: *const MemoryAreas) -> &[u8] {
    let mem = unsafe { &*mem_ptr };
    match ip {
      0x0000..=0x3fff => &mem.rom[ip..0x4000],
      0x4000..=0x7fff => {
        let bank_start = mem.rom_bank * 0x4000;
        let bank_end = bank_start + 0x4000;
        let offset = (ip & 0x3fff) + bank_start;
        &mem.rom[offset..bank_end]
      },
      0xc000..=0xcfff => &mem.work_ram[(ip & 0xfff)..0x1000],
      0xd000..=0xdfff => {
        let bank_start = mem.wram_bank * 0x1000;
        let bank_end = bank_start + 0x1000;
        let offset = (ip & 0xfff) + bank_start;
        &mem.work_ram[offset..bank_end]
      },
      _ => panic!(),
    }
  }

  pub fn translate_code_block(&mut self, code: &Box<[u8]>, ip: usize, mem: *const MemoryAreas) -> usize {
    let mut write_cursor = self.write_cursor;
    let starting_offset = write_cursor;

    let emitter = Emitter::new(mem);
    let available_length = {
      self.exec_memory.make_writable();
      let translated = self.exec_memory.get_memory_area_mut();
      write_cursor += emitter.encode_prelude(&mut translated[write_cursor..]);
      translated.len()
    };

    let mut block_ended = false;
    let mut index = ip;
    while !block_ended {
      let code_slice = self.get_executable_memory_segment(index, mem);
      if code_slice.len() < 1 {
        break;
      }
      let (next_op, length, _cycles) = decode(code_slice);
      index += length;
      block_ended = next_op.is_block_end();
      let translated = self.exec_memory.get_memory_area_mut();
      let written = emitter.encode_op(next_op, length, &mut translated[write_cursor..]);
      write_cursor += written;
    }

    
    #[cfg(feature = "dump_disassembly")]
    {
      let code_slice = match ip {
        0..=0x7fff => &code[ip..index],
        0xc000..=0xdfff => unsafe {
          &(*mem).work_ram[(ip - 0xc000)..(index - 0xc000)]
        },
        _ => panic!(),
      };
      let disassembly = crate::debug::disassembly::disassemble(ip as u16, code_slice);
      for instr in disassembly.iter() {
        println!("{}", instr);
      }
      println!("  ==  ");
    }
    
    {
      let translated = self.exec_memory.get_memory_area_mut();
      write_cursor += emitter.encode_epilogue(&mut translated[write_cursor..]);
    }

    self.exec_memory.make_executable();
    self.write_cursor = write_cursor;

    let bytes_translated = index - ip;
    self.insert_code_block(ip, starting_offset, write_cursor - starting_offset, bytes_translated);

    let space_remaining = available_length - write_cursor;
    if space_remaining < MEMORY_MINIMUM_SIZE {
      println!("Running out of space, only {} bytes left", space_remaining);
    }

    starting_offset
  }

  fn insert_code_block(&mut self, ip: usize, offset: usize, length: usize, bytes_translated: usize) {
    let region = self.code_blocks
      .get_region_mut(ip as u16)
      .expect("Cannot cache code in this region");
    region.insert(
      ip as u16,
      CodeBlock {
        offset,
        length,
        bytes_translated,
      },
    );
  }

  pub fn call(&self, offset: usize, registers: &mut Registers) -> u8 {
    let func_pointer = (self.get_memory_start_address() + offset) as *const ();
    let func: extern "sysv64" fn(*const Registers) -> u8 = unsafe {
      std::mem::transmute(func_pointer)
    };
    func(registers as *const Registers)
  }
}
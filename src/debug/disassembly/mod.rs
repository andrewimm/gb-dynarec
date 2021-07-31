use std::fmt::format;

use crate::decoder;

pub struct Instruction(pub u16, pub String);

impl std::fmt::Display for Instruction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("{:#06X}  {}", self.0, self.1))
  }
}

pub fn disassemble(initial_addr: u16, instructions: &[u8]) -> Vec<Instruction> {
  let mut output = Vec::new();
  let mut cursor = 0;
  let mut addr = initial_addr;
  while cursor < instructions.len() {
    let (op, length, _) = decoder::decode(&instructions[cursor..]);
    output.push(Instruction(addr, format(format_args!("{}", op))));
    addr = addr.wrapping_add(length as u16);
    cursor += length;
  }

  output
}
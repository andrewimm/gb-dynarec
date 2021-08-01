use crate::decoder;

pub struct Instruction {
  address: u16,
  bytes: [u8; 4],
  length: usize,
  text: String,
}

impl std::fmt::Display for Instruction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut formatted = format!("{:#06X}  ", self.address);
    for i in 0..self.length {
      formatted.push_str(format!("{:02X} ", self.bytes[i]).as_str());
    }
    for _ in self.length..4 {
      formatted.push_str("   ");
    }
    formatted.push_str(self.text.as_str());
    f.write_str(formatted.as_str())
  }
}

pub fn disassemble(initial_addr: u16, instructions: &[u8]) -> Vec<Instruction> {
  let mut output = Vec::new();
  let mut cursor = 0;
  let mut address = initial_addr;
  while cursor < instructions.len() {
    let (op, length, _) = decoder::decode(&instructions[cursor..]);
    let mut bytes: [u8; 4] = [0; 4];
    for i in 0..length {
      bytes[i] = instructions[cursor + i];
    }
    output.push(Instruction {
      address,
      bytes,
      length,
      text: op.to_string(),
    });
    address = address.wrapping_add(length as u16);
    cursor += length;
  }

  output
}
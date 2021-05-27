pub mod ops;

use ops::{IndirectLocation, JumpCondition, Op, Register8, Register16, Source8};

pub fn decode(instructions: &[u8]) -> (Op, usize, usize) {
  match instructions[0] {
    0x00 => (Op::NoOp, 1, 4),
    0x01 => {
      let value = read_u16(&instructions[1..]);
      let op = Op::Load16(Register16::BC, value);
      (op, 3, 12)
    },
    0x02 => (Op::LoadToIndirect(IndirectLocation::BC, Source8::A), 1, 8),
    0x03 => (Op::Increment16(Register16::BC), 1, 8),
    0x04 => (Op::Increment8(Register8::B), 1, 4),
    0x05 => (Op::Decrement8(Register8::B), 1, 4),
    0x06 => {
      let value = instructions[1];
      let op = Op::Load8Immediate(Register8::B, value);
      (op, 2, 8)
    },
    0x07 => (Op::RotateLeftCarryA, 1, 4),
    0x08 => {
      let value = read_u16(&instructions[1..]);
      let op = Op::LoadStackPointerToMemory(value);
      (op, 3, 20)
    }
    0x09 => (Op::AddHL(Register16::BC), 1, 8),
    0x0a => (Op::LoadFromIndirect(Register8::A, IndirectLocation::BC), 1, 8),
    0x0b => (Op::Decrement16(Register16::BC), 1, 8),
    0x0c => (Op::Increment8(Register8::C), 1, 4),
    0x0d => (Op::Decrement8(Register8::C), 1, 4),
    0x0e => {
      let value = instructions[1];
      let op = Op::Load8Immediate(Register8::C, value);
      (op, 2, 8)
    },
    0x0f => (Op::RotateRightCarryA, 1, 4),

    0x10 => (Op::Stop, 2, 4), // Consumes 2 bytes, expects second to be 0x00
    0x11 => {
      let value = read_u16(&instructions[1..]);
      let op = Op::Load16(Register16::DE, value);
      (op, 3, 12)
    },
    0x12 => (Op::LoadToIndirect(IndirectLocation::DE, Source8::A), 1, 8),
    0x13 => (Op::Increment16(Register16::DE), 1, 8),
    0x14 => (Op::Increment8(Register8::D), 1, 4),
    0x15 => (Op::Decrement8(Register8::D), 1, 4),
    0x16 => {
      let value = instructions[1];
      let op = Op::Load8Immediate(Register8::D, value);
      (op, 2, 8)
    },
    0x17 => (Op::RotateLeftA, 1, 4),
    0x18 => {
      let offset = instructions[1] as i8;
      let op = Op::JumpRelative(JumpCondition::Always, offset);
      (op, 2, 12)
    },
    0x19 => (Op::AddHL(Register16::DE), 1, 8),
    0x1a => (Op::LoadFromIndirect(Register8::A, IndirectLocation::DE), 1, 8),
    0x1b => (Op::Decrement16(Register16::DE), 1, 8),
    0x1c => (Op::Increment8(Register8::E), 1, 4),
    0x1d => (Op::Decrement8(Register8::E), 1, 4),
    0x1e => {
      let value = instructions[1];
      let op = Op::Load8Immediate(Register8::E, value);
      (op, 2, 8)
    },
    0x1f => (Op::RotateRightA, 1, 4),

    0x20 => {
      let offset = instructions[1] as i8;
      let op = Op::JumpRelative(JumpCondition::NonZero, offset);
      (op, 2, 12)
    },
    0x21 => {
      let value = read_u16(&instructions[1..]);
      let op = Op::Load16(Register16::HL, value);
      (op, 3, 12)
    },
    0x22 => (Op::LoadToIndirect(IndirectLocation::HLIncrement, Source8::A), 1, 8),
    0x23 => (Op::Increment16(Register16::HL), 1, 8),
    0x24 => (Op::Increment8(Register8::H), 1, 4),
    0x25 => (Op::Decrement8(Register8::H), 1, 4),
    0x26 => {
      let value = instructions[1];
      let op = Op::Load8Immediate(Register8::H, value);
      (op, 2, 8)
    },
    0x27 => (Op::DAA, 1, 4),
    0x28 => {
      let offset = instructions[1] as i8;
      let op = Op::JumpRelative(JumpCondition::Zero, offset);
      (op, 2, 12)
    },
    0x29 => (Op::AddHL(Register16::HL), 1, 8),
    0x2a => (Op::LoadFromIndirect(Register8::A, IndirectLocation::HLIncrement), 1, 8),
    0x2b => (Op::Decrement16(Register16::HL), 1, 8),
    0x2c => (Op::Increment8(Register8::L), 1, 4),
    0x2d => (Op::Decrement8(Register8::L), 1, 4),
    0x2e => {
      let value = instructions[1];
      let op = Op::Load8Immediate(Register8::L, value);
      (op, 2, 8)
    },
    0x2f => (Op::ComplementA, 1, 4),

    0x30 => {
      let offset = instructions[1] as i8;
      let op = Op::JumpRelative(JumpCondition::NoCarry, offset);
      (op, 2, 12)
    },
    0x31 => {
      let value = read_u16(&instructions[1..]);
      let op = Op::Load16(Register16::SP, value);
      (op, 3, 12)
    },
    0x32 => (Op::LoadToIndirect(IndirectLocation::HLDecrement, Source8::A), 1, 8),
    0x33 => (Op::Increment16(Register16::SP), 1, 8),
    0x34 => (Op::IncrementHLIndirect, 1, 12),
    0x35 => (Op::DecrementHLIndirect, 1, 12),
    0x36 => {
      let value = instructions[1];
      let op = Op::LoadToIndirect(IndirectLocation::HL, Source8::Literal(value));
      (op, 2, 12)
    },
    0x37 => (Op::SetCarryFlag, 1, 4),
    0x38 => {
      let offset = instructions[1] as i8;
      let op = Op::JumpRelative(JumpCondition::Carry, offset);
      (op, 2, 12)
    },
    0x39 => (Op::AddHL(Register16::SP), 1, 8),
    0x3a => (Op::LoadFromIndirect(Register8::A, IndirectLocation::HLDecrement), 1, 8),
    0x3c => (Op::Increment8(Register8::A), 1, 4),
    0x3d => (Op::Decrement8(Register8::A), 1, 4),
    0x3e => {
      let value = instructions[1];
      let op = Op::Load8Immediate(Register8::A, value);
      (op, 2, 8)
    },
    0x3f => (Op::ComplementCarryFlag, 1, 4),

    0x40 => (Op::Load8(Register8::B, Register8::B), 1, 4),
    0x41 => (Op::Load8(Register8::B, Register8::C), 1, 4),
    0x42 => (Op::Load8(Register8::B, Register8::D), 1, 4),
    0x43 => (Op::Load8(Register8::B, Register8::E), 1, 4),
    0x44 => (Op::Load8(Register8::B, Register8::H), 1, 4),
    0x45 => (Op::Load8(Register8::B, Register8::L), 1, 4),
    0x46 => (Op::LoadFromIndirect(Register8::B, IndirectLocation::HL), 1, 8),
    0x47 => (Op::Load8(Register8::B, Register8::A), 1, 4),
    0x48 => (Op::Load8(Register8::C, Register8::B), 1, 4),
    0x49 => (Op::Load8(Register8::C, Register8::C), 1, 4),
    0x4a => (Op::Load8(Register8::C, Register8::D), 1, 4),
    0x4b => (Op::Load8(Register8::C, Register8::E), 1, 4),
    0x4c => (Op::Load8(Register8::C, Register8::H), 1, 4),
    0x4d => (Op::Load8(Register8::C, Register8::L), 1, 4),
    0x4e => (Op::LoadFromIndirect(Register8::C, IndirectLocation::HL), 1, 8),
    0x4f => (Op::Load8(Register8::C, Register8::A), 1, 4),

    0x50 => (Op::Load8(Register8::D, Register8::B), 1, 4),
    0x51 => (Op::Load8(Register8::D, Register8::C), 1, 4),
    0x52 => (Op::Load8(Register8::D, Register8::D), 1, 4),
    0x53 => (Op::Load8(Register8::D, Register8::E), 1, 4),
    0x54 => (Op::Load8(Register8::D, Register8::H), 1, 4),
    0x55 => (Op::Load8(Register8::D, Register8::L), 1, 4),
    0x56 => (Op::LoadFromIndirect(Register8::D, IndirectLocation::HL), 1, 8),
    0x57 => (Op::Load8(Register8::D, Register8::A), 1, 4),
    0x58 => (Op::Load8(Register8::E, Register8::B), 1, 4),
    0x59 => (Op::Load8(Register8::E, Register8::C), 1, 4),
    0x5a => (Op::Load8(Register8::E, Register8::D), 1, 4),
    0x5b => (Op::Load8(Register8::E, Register8::E), 1, 4),
    0x5c => (Op::Load8(Register8::E, Register8::H), 1, 4),
    0x5d => (Op::Load8(Register8::E, Register8::L), 1, 4),
    0x5e => (Op::LoadFromIndirect(Register8::E, IndirectLocation::HL), 1, 8),
    0x5f => (Op::Load8(Register8::E, Register8::A), 1, 4),

    0x60 => (Op::Load8(Register8::H, Register8::B), 1, 4),
    0x61 => (Op::Load8(Register8::H, Register8::C), 1, 4),
    0x62 => (Op::Load8(Register8::H, Register8::D), 1, 4),
    0x63 => (Op::Load8(Register8::H, Register8::E), 1, 4),
    0x64 => (Op::Load8(Register8::H, Register8::H), 1, 4),
    0x65 => (Op::Load8(Register8::H, Register8::L), 1, 4),
    0x66 => (Op::LoadFromIndirect(Register8::H, IndirectLocation::HL), 1, 8),
    0x67 => (Op::Load8(Register8::H, Register8::A), 1, 4),
    0x68 => (Op::Load8(Register8::L, Register8::B), 1, 4),
    0x69 => (Op::Load8(Register8::L, Register8::C), 1, 4),
    0x6a => (Op::Load8(Register8::L, Register8::D), 1, 4),
    0x6b => (Op::Load8(Register8::L, Register8::E), 1, 4),
    0x6c => (Op::Load8(Register8::L, Register8::H), 1, 4),
    0x6d => (Op::Load8(Register8::L, Register8::L), 1, 4),
    0x6e => (Op::LoadFromIndirect(Register8::L, IndirectLocation::HL), 1, 8),
    0x6f => (Op::Load8(Register8::L, Register8::A), 1, 4),

    0x70 => (Op::LoadToIndirect(IndirectLocation::HL, Source8::B), 1, 8),
    0x71 => (Op::LoadToIndirect(IndirectLocation::HL, Source8::C), 1, 8),
    0x72 => (Op::LoadToIndirect(IndirectLocation::HL, Source8::D), 1, 8),
    0x73 => (Op::LoadToIndirect(IndirectLocation::HL, Source8::E), 1, 8),
    0x74 => (Op::LoadToIndirect(IndirectLocation::HL, Source8::H), 1, 8),
    0x75 => (Op::LoadToIndirect(IndirectLocation::HL, Source8::L), 1, 8),
    0x76 => (Op::Halt, 1, 4),
    0x77 => (Op::LoadToIndirect(IndirectLocation::HL, Source8::A), 1, 8),
    0x78 => (Op::Load8(Register8::A, Register8::B), 1, 4),
    0x79 => (Op::Load8(Register8::A, Register8::C), 1, 4),
    0x7a => (Op::Load8(Register8::A, Register8::D), 1, 4),
    0x7b => (Op::Load8(Register8::A, Register8::E), 1, 4),
    0x7c => (Op::Load8(Register8::A, Register8::H), 1, 4),
    0x7d => (Op::Load8(Register8::A, Register8::L), 1, 4),
    0x7e => (Op::LoadFromIndirect(Register8::A, IndirectLocation::HL), 1, 8),
    0x7f => (Op::Load8(Register8::A, Register8::A), 1, 4),

    0x80 => (Op::Add8(Register8::A, Register8::B), 1, 4),
    0x81 => (Op::Add8(Register8::A, Register8::C), 1, 4),
    0x82 => (Op::Add8(Register8::A, Register8::D), 1, 4),
    0x83 => (Op::Add8(Register8::A, Register8::E), 1, 4),
    0x84 => (Op::Add8(Register8::A, Register8::H), 1, 4),
    0x85 => (Op::Add8(Register8::A, Register8::L), 1, 4),

    0x87 => (Op::Add8(Register8::A, Register8::A), 1, 4),
    0x88 => (Op::AddWithCarry8(Register8::A, Register8::B), 1, 4),
    0x89 => (Op::AddWithCarry8(Register8::A, Register8::C), 1, 4),
    0x8a => (Op::AddWithCarry8(Register8::A, Register8::D), 1, 4),
    0x8b => (Op::AddWithCarry8(Register8::A, Register8::E), 1, 4),
    0x8c => (Op::AddWithCarry8(Register8::A, Register8::H), 1, 4),
    0x8d => (Op::AddWithCarry8(Register8::A, Register8::L), 1, 4),

    0x8f => (Op::AddWithCarry8(Register8::A, Register8::A), 1, 4),

    0x90 => (Op::Sub8(Register8::A, Register8::B), 1, 4),
    0x91 => (Op::Sub8(Register8::A, Register8::C), 1, 4),
    0x92 => (Op::Sub8(Register8::A, Register8::D), 1, 4),
    0x93 => (Op::Sub8(Register8::A, Register8::E), 1, 4),
    0x94 => (Op::Sub8(Register8::A, Register8::H), 1, 4),
    0x95 => (Op::Sub8(Register8::A, Register8::L), 1, 4),

    0x97 => (Op::Sub8(Register8::A, Register8::A), 1, 4),
    0x98 => (Op::SubWithCarry8(Register8::A, Register8::B), 1, 4),
    0x99 => (Op::SubWithCarry8(Register8::A, Register8::C), 1, 4),
    0x9a => (Op::SubWithCarry8(Register8::A, Register8::D), 1, 4),
    0x9b => (Op::SubWithCarry8(Register8::A, Register8::E), 1, 4),
    0x9c => (Op::SubWithCarry8(Register8::A, Register8::H), 1, 4),
    0x9d => (Op::SubWithCarry8(Register8::A, Register8::L), 1, 4),

    0x9f => (Op::SubWithCarry8(Register8::A, Register8::A), 1, 4),

    0xa0 => (Op::And8(Register8::A, Register8::B), 1, 4),
    0xa1 => (Op::And8(Register8::A, Register8::C), 1, 4),
    0xa2 => (Op::And8(Register8::A, Register8::D), 1, 4),
    0xa3 => (Op::And8(Register8::A, Register8::E), 1, 4),
    0xa4 => (Op::And8(Register8::A, Register8::H), 1, 4),
    0xa5 => (Op::And8(Register8::A, Register8::L), 1, 4),

    0xa7 => (Op::And8(Register8::A, Register8::A), 1, 4),
    0xa8 => (Op::Xor8(Register8::A, Register8::B), 1, 4),
    0xa9 => (Op::Xor8(Register8::A, Register8::C), 1, 4),
    0xaa => (Op::Xor8(Register8::A, Register8::D), 1, 4),
    0xab => (Op::Xor8(Register8::A, Register8::E), 1, 4),
    0xac => (Op::Xor8(Register8::A, Register8::H), 1, 4),
    0xad => (Op::Xor8(Register8::A, Register8::L), 1, 4),

    0xaf => (Op::Xor8(Register8::A, Register8::A), 1, 4),

    0xb0 => (Op::Or8(Register8::A, Register8::B), 1, 4),
    0xb1 => (Op::Or8(Register8::A, Register8::C), 1, 4),
    0xb2 => (Op::Or8(Register8::A, Register8::D), 1, 4),
    0xb3 => (Op::Or8(Register8::A, Register8::E), 1, 4),
    0xb4 => (Op::Or8(Register8::A, Register8::H), 1, 4),
    0xb5 => (Op::Or8(Register8::A, Register8::L), 1, 4),

    0xb7 => (Op::Or8(Register8::A, Register8::A), 1, 4),
    0xb8 => (Op::Compare8(Register8::A, Register8::B), 1, 4),
    0xb9 => (Op::Compare8(Register8::A, Register8::C), 1, 4),
    0xba => (Op::Compare8(Register8::A, Register8::D), 1, 4),
    0xbb => (Op::Compare8(Register8::A, Register8::E), 1, 4),
    0xbc => (Op::Compare8(Register8::A, Register8::H), 1, 4),
    0xbd => (Op::Compare8(Register8::A, Register8::L), 1, 4),

    0xbf => (Op::Compare8(Register8::A, Register8::A), 1, 4),

    0xc2 => {
      let addr = read_u16(&instructions[1..]);
      let op = Op::Jump(JumpCondition::NonZero, addr);
      (op, 3, 12)
    },
    0xc3 => {
      let addr = read_u16(&instructions[1..]);
      let op = Op::Jump(JumpCondition::Always, addr);
      (op, 3, 16)
    },
    0xc4 => {
      let addr = read_u16(&instructions[1..]);
      let op = Op::Call(JumpCondition::NonZero, addr);
      (op, 3, 12)
    },

    0xc6 => {
      let value = instructions[1];
      let op = Op::AddAbsolute8(value);
      (op, 2, 8)
    },

    0xca => {
      let addr = read_u16(&instructions[1..]);
      let op = Op::Jump(JumpCondition::Zero, addr);
      (op, 3, 12)
    },
    0xcb => decode_cb(&instructions[1..]),

    0xcd => {
      let addr = read_u16(&instructions[1..]);
      let op = Op::Call(JumpCondition::Always, addr);
      (op, 3, 24)
    },

    _ => (Op::Invalid(instructions[0]), 1, 4),
  }
}

fn decode_cb(instructions: &[u8]) -> (Op, usize, usize) {
  match instructions[0] {
    0x00 => (Op::RotateLeftCarry(Register8::B), 2, 8),
    0x01 => (Op::RotateLeftCarry(Register8::C), 2, 8),
    0x02 => (Op::RotateLeftCarry(Register8::D), 2, 8),
    0x03 => (Op::RotateLeftCarry(Register8::E), 2, 8),
    0x04 => (Op::RotateLeftCarry(Register8::H), 2, 8),
    0x05 => (Op::RotateLeftCarry(Register8::L), 2, 8),

    0x07 => (Op::RotateLeftCarry(Register8::A), 2, 8),
    0x08 => (Op::RotateRightCarry(Register8::B), 2, 8),
    0x09 => (Op::RotateRightCarry(Register8::C), 2, 8),
    0x0a => (Op::RotateRightCarry(Register8::D), 2, 8),
    0x0b => (Op::RotateRightCarry(Register8::E), 2, 8),
    0x0c => (Op::RotateRightCarry(Register8::H), 2, 8),
    0x0d => (Op::RotateRightCarry(Register8::L), 2, 8),

    0x0f => (Op::RotateRightCarry(Register8::A), 2, 8),

    0x10 => (Op::RotateLeft(Register8::B), 2, 8),
    0x11 => (Op::RotateLeft(Register8::C), 2, 8),
    0x12 => (Op::RotateLeft(Register8::D), 2, 8),
    0x13 => (Op::RotateLeft(Register8::E), 2, 8),
    0x14 => (Op::RotateLeft(Register8::H), 2, 8),
    0x15 => (Op::RotateLeft(Register8::L), 2, 8),

    0x17 => (Op::RotateLeft(Register8::A), 2, 8),
    0x18 => (Op::RotateRight(Register8::B), 2, 8),
    0x19 => (Op::RotateRight(Register8::C), 2, 8),
    0x1a => (Op::RotateRight(Register8::D), 2, 8),
    0x1b => (Op::RotateRight(Register8::E), 2, 8),
    0x1c => (Op::RotateRight(Register8::H), 2, 8),
    0x1d => (Op::RotateRight(Register8::L), 2, 8),

    0x1f => (Op::RotateRight(Register8::A), 2, 8),

    0x30 => (Op::Swap(Register8::B), 2, 8),
    0x31 => (Op::Swap(Register8::C), 2, 8),
    0x32 => (Op::Swap(Register8::D), 2, 8),
    0x33 => (Op::Swap(Register8::E), 2, 8),
    0x34 => (Op::Swap(Register8::H), 2, 8),
    0x35 => (Op::Swap(Register8::L), 2, 8),

    0x37 => (Op::Swap(Register8::A), 2, 8),

    0x40 => (Op::BitTest(Register8::B, 0x01), 2, 8),
    0x41 => (Op::BitTest(Register8::C, 0x01), 2, 8),
    0x42 => (Op::BitTest(Register8::D, 0x01), 2, 8),
    0x43 => (Op::BitTest(Register8::E, 0x01), 2, 8),
    0x44 => (Op::BitTest(Register8::H, 0x01), 2, 8),
    0x45 => (Op::BitTest(Register8::L, 0x01), 2, 8),

    0x47 => (Op::BitTest(Register8::A, 0x01), 2, 8),
    0x48 => (Op::BitTest(Register8::B, 0x02), 2, 8),
    0x49 => (Op::BitTest(Register8::C, 0x02), 2, 8),
    0x4a => (Op::BitTest(Register8::D, 0x02), 2, 8),
    0x4b => (Op::BitTest(Register8::E, 0x02), 2, 8),
    0x4c => (Op::BitTest(Register8::H, 0x02), 2, 8),
    0x4d => (Op::BitTest(Register8::L, 0x02), 2, 8),

    0x4f => (Op::BitTest(Register8::A, 0x02), 2, 8),

    0x50 => (Op::BitTest(Register8::B, 0x04), 2, 8),
    0x51 => (Op::BitTest(Register8::C, 0x04), 2, 8),
    0x52 => (Op::BitTest(Register8::D, 0x04), 2, 8),
    0x53 => (Op::BitTest(Register8::E, 0x04), 2, 8),
    0x54 => (Op::BitTest(Register8::H, 0x04), 2, 8),
    0x55 => (Op::BitTest(Register8::L, 0x04), 2, 8),

    0x57 => (Op::BitTest(Register8::A, 0x04), 2, 8),
    0x58 => (Op::BitTest(Register8::B, 0x08), 2, 8),
    0x59 => (Op::BitTest(Register8::C, 0x08), 2, 8),
    0x5a => (Op::BitTest(Register8::D, 0x08), 2, 8),
    0x5b => (Op::BitTest(Register8::E, 0x08), 2, 8),
    0x5c => (Op::BitTest(Register8::H, 0x08), 2, 8),
    0x5d => (Op::BitTest(Register8::L, 0x08), 2, 8),

    0x5f => (Op::BitTest(Register8::A, 0x08), 2, 8),

    0x60 => (Op::BitTest(Register8::B, 0x10), 2, 8),
    0x61 => (Op::BitTest(Register8::C, 0x10), 2, 8),
    0x62 => (Op::BitTest(Register8::D, 0x10), 2, 8),
    0x63 => (Op::BitTest(Register8::E, 0x10), 2, 8),
    0x64 => (Op::BitTest(Register8::H, 0x10), 2, 8),
    0x65 => (Op::BitTest(Register8::L, 0x10), 2, 8),

    0x67 => (Op::BitTest(Register8::A, 0x10), 2, 8),
    0x68 => (Op::BitTest(Register8::B, 0x20), 2, 8),
    0x69 => (Op::BitTest(Register8::C, 0x20), 2, 8),
    0x6a => (Op::BitTest(Register8::D, 0x20), 2, 8),
    0x6b => (Op::BitTest(Register8::E, 0x20), 2, 8),
    0x6c => (Op::BitTest(Register8::H, 0x20), 2, 8),
    0x6d => (Op::BitTest(Register8::L, 0x20), 2, 8),

    0x6f => (Op::BitTest(Register8::A, 0x20), 2, 8),

    0x70 => (Op::BitTest(Register8::B, 0x40), 2, 8),
    0x71 => (Op::BitTest(Register8::C, 0x40), 2, 8),
    0x72 => (Op::BitTest(Register8::D, 0x40), 2, 8),
    0x73 => (Op::BitTest(Register8::E, 0x40), 2, 8),
    0x74 => (Op::BitTest(Register8::H, 0x40), 2, 8),
    0x75 => (Op::BitTest(Register8::L, 0x40), 2, 8),

    0x77 => (Op::BitTest(Register8::A, 0x40), 2, 8),
    0x78 => (Op::BitTest(Register8::B, 0x80), 2, 8),
    0x79 => (Op::BitTest(Register8::C, 0x80), 2, 8),
    0x7a => (Op::BitTest(Register8::D, 0x80), 2, 8),
    0x7b => (Op::BitTest(Register8::E, 0x80), 2, 8),
    0x7c => (Op::BitTest(Register8::H, 0x80), 2, 8),
    0x7d => (Op::BitTest(Register8::L, 0x80), 2, 8),

    0x7f => (Op::BitTest(Register8::A, 0x80), 2, 8),

    0x80 => (Op::BitClear(Register8::B, 0x01), 2, 8),
    0x81 => (Op::BitClear(Register8::C, 0x01), 2, 8),
    0x82 => (Op::BitClear(Register8::D, 0x01), 2, 8),
    0x83 => (Op::BitClear(Register8::E, 0x01), 2, 8),
    0x84 => (Op::BitClear(Register8::H, 0x01), 2, 8),
    0x85 => (Op::BitClear(Register8::L, 0x01), 2, 8),

    0x87 => (Op::BitClear(Register8::A, 0x01), 2, 8),
    0x88 => (Op::BitClear(Register8::B, 0x02), 2, 8),
    0x89 => (Op::BitClear(Register8::C, 0x02), 2, 8),
    0x8a => (Op::BitClear(Register8::D, 0x02), 2, 8),
    0x8b => (Op::BitClear(Register8::E, 0x02), 2, 8),
    0x8c => (Op::BitClear(Register8::H, 0x02), 2, 8),
    0x8d => (Op::BitClear(Register8::L, 0x02), 2, 8),

    0x8f => (Op::BitClear(Register8::A, 0x02), 2, 8),

    0x90 => (Op::BitClear(Register8::B, 0x04), 2, 8),
    0x91 => (Op::BitClear(Register8::C, 0x04), 2, 8),
    0x92 => (Op::BitClear(Register8::D, 0x04), 2, 8),
    0x93 => (Op::BitClear(Register8::E, 0x04), 2, 8),
    0x94 => (Op::BitClear(Register8::H, 0x04), 2, 8),
    0x95 => (Op::BitClear(Register8::L, 0x04), 2, 8),

    0x97 => (Op::BitClear(Register8::A, 0x04), 2, 8),
    0x98 => (Op::BitClear(Register8::B, 0x08), 2, 8),
    0x99 => (Op::BitClear(Register8::C, 0x08), 2, 8),
    0x9a => (Op::BitClear(Register8::D, 0x08), 2, 8),
    0x9b => (Op::BitClear(Register8::E, 0x08), 2, 8),
    0x9c => (Op::BitClear(Register8::H, 0x08), 2, 8),
    0x9d => (Op::BitClear(Register8::L, 0x08), 2, 8),

    0x9f => (Op::BitClear(Register8::A, 0x08), 2, 8),

    0xa0 => (Op::BitClear(Register8::B, 0x10), 2, 8),
    0xa1 => (Op::BitClear(Register8::C, 0x10), 2, 8),
    0xa2 => (Op::BitClear(Register8::D, 0x10), 2, 8),
    0xa3 => (Op::BitClear(Register8::E, 0x10), 2, 8),
    0xa4 => (Op::BitClear(Register8::H, 0x10), 2, 8),
    0xa5 => (Op::BitClear(Register8::L, 0x10), 2, 8),

    0xa7 => (Op::BitClear(Register8::A, 0x10), 2, 8),
    0xa8 => (Op::BitClear(Register8::B, 0x20), 2, 8),
    0xa9 => (Op::BitClear(Register8::C, 0x20), 2, 8),
    0xaa => (Op::BitClear(Register8::D, 0x20), 2, 8),
    0xab => (Op::BitClear(Register8::E, 0x20), 2, 8),
    0xac => (Op::BitClear(Register8::H, 0x20), 2, 8),
    0xad => (Op::BitClear(Register8::L, 0x20), 2, 8),

    0xaf => (Op::BitClear(Register8::A, 0x20), 2, 8),

    0xb0 => (Op::BitClear(Register8::B, 0x40), 2, 8),
    0xb1 => (Op::BitClear(Register8::C, 0x40), 2, 8),
    0xb2 => (Op::BitClear(Register8::D, 0x40), 2, 8),
    0xb3 => (Op::BitClear(Register8::E, 0x40), 2, 8),
    0xb4 => (Op::BitClear(Register8::H, 0x40), 2, 8),
    0xb5 => (Op::BitClear(Register8::L, 0x40), 2, 8),

    0xb7 => (Op::BitClear(Register8::A, 0x40), 2, 8),
    0xb8 => (Op::BitClear(Register8::B, 0x80), 2, 8),
    0xb9 => (Op::BitClear(Register8::C, 0x80), 2, 8),
    0xba => (Op::BitClear(Register8::D, 0x80), 2, 8),
    0xbb => (Op::BitClear(Register8::E, 0x80), 2, 8),
    0xbc => (Op::BitClear(Register8::H, 0x80), 2, 8),
    0xbd => (Op::BitClear(Register8::L, 0x80), 2, 8),

    0xbf => (Op::BitClear(Register8::A, 0x80), 2, 8),

    0xc0 => (Op::BitSet(Register8::B, 0x01), 2, 8),
    0xc1 => (Op::BitSet(Register8::C, 0x01), 2, 8),
    0xc2 => (Op::BitSet(Register8::D, 0x01), 2, 8),
    0xc3 => (Op::BitSet(Register8::E, 0x01), 2, 8),
    0xc4 => (Op::BitSet(Register8::H, 0x01), 2, 8),
    0xc5 => (Op::BitSet(Register8::L, 0x01), 2, 8),

    0xc7 => (Op::BitSet(Register8::A, 0x01), 2, 8),
    0xc8 => (Op::BitSet(Register8::B, 0x02), 2, 8),
    0xc9 => (Op::BitSet(Register8::C, 0x02), 2, 8),
    0xca => (Op::BitSet(Register8::D, 0x02), 2, 8),
    0xcb => (Op::BitSet(Register8::E, 0x02), 2, 8),
    0xcc => (Op::BitSet(Register8::H, 0x02), 2, 8),
    0xcd => (Op::BitSet(Register8::L, 0x02), 2, 8),

    0xcf => (Op::BitSet(Register8::A, 0x02), 2, 8),

    0xd0 => (Op::BitSet(Register8::B, 0x04), 2, 8),
    0xd1 => (Op::BitSet(Register8::C, 0x04), 2, 8),
    0xd2 => (Op::BitSet(Register8::D, 0x04), 2, 8),
    0xd3 => (Op::BitSet(Register8::E, 0x04), 2, 8),
    0xd4 => (Op::BitSet(Register8::H, 0x04), 2, 8),
    0xd5 => (Op::BitSet(Register8::L, 0x04), 2, 8),

    0xd7 => (Op::BitSet(Register8::A, 0x04), 2, 8),
    0xd8 => (Op::BitSet(Register8::B, 0x08), 2, 8),
    0xd9 => (Op::BitSet(Register8::C, 0x08), 2, 8),
    0xda => (Op::BitSet(Register8::D, 0x08), 2, 8),
    0xdb => (Op::BitSet(Register8::E, 0x08), 2, 8),
    0xdc => (Op::BitSet(Register8::H, 0x08), 2, 8),
    0xdd => (Op::BitSet(Register8::L, 0x08), 2, 8),

    0xdf => (Op::BitSet(Register8::A, 0x08), 2, 8),

    0xe0 => (Op::BitSet(Register8::B, 0x10), 2, 8),
    0xe1 => (Op::BitSet(Register8::C, 0x10), 2, 8),
    0xe2 => (Op::BitSet(Register8::D, 0x10), 2, 8),
    0xe3 => (Op::BitSet(Register8::E, 0x10), 2, 8),
    0xe4 => (Op::BitSet(Register8::H, 0x10), 2, 8),
    0xe5 => (Op::BitSet(Register8::L, 0x10), 2, 8),

    0xe7 => (Op::BitSet(Register8::A, 0x10), 2, 8),
    0xe8 => (Op::BitSet(Register8::B, 0x20), 2, 8),
    0xe9 => (Op::BitSet(Register8::C, 0x20), 2, 8),
    0xea => (Op::BitSet(Register8::D, 0x20), 2, 8),
    0xeb => (Op::BitSet(Register8::E, 0x20), 2, 8),
    0xec => (Op::BitSet(Register8::H, 0x20), 2, 8),
    0xed => (Op::BitSet(Register8::L, 0x20), 2, 8),

    0xef => (Op::BitSet(Register8::A, 0x20), 2, 8),

    0xf0 => (Op::BitSet(Register8::B, 0x40), 2, 8),
    0xf1 => (Op::BitSet(Register8::C, 0x40), 2, 8),
    0xf2 => (Op::BitSet(Register8::D, 0x40), 2, 8),
    0xf3 => (Op::BitSet(Register8::E, 0x40), 2, 8),
    0xf4 => (Op::BitSet(Register8::H, 0x40), 2, 8),
    0xf5 => (Op::BitSet(Register8::L, 0x40), 2, 8),

    0xf7 => (Op::BitSet(Register8::A, 0x40), 2, 8),
    0xf8 => (Op::BitSet(Register8::B, 0x80), 2, 8),
    0xf9 => (Op::BitSet(Register8::C, 0x80), 2, 8),
    0xfa => (Op::BitSet(Register8::D, 0x80), 2, 8),
    0xfb => (Op::BitSet(Register8::E, 0x80), 2, 8),
    0xfc => (Op::BitSet(Register8::H, 0x80), 2, 8),
    0xfd => (Op::BitSet(Register8::L, 0x80), 2, 8),

    0xff => (Op::BitSet(Register8::A, 0x80), 2, 8),

    _ => panic!("Unsupported CB Op: {:X}", instructions[0]),
  }
}

fn read_u16(instructions: &[u8]) -> u16 {
  let low = instructions[0] as u16;
  let high = instructions[1] as u16;
  (high << 8) | low
}
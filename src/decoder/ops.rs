pub enum Op {
  Invalid(u8),
  NoOp,
  Stop,
  Halt,
  Load16(Register16, u16),
  LoadToIndirect(IndirectLocation, Register8),
  LoadImmediateToHLIndirect(u8),
  LoadFromIndirect(Register8, IndirectLocation),
  Increment16(Register16),
  Decrement16(Register16),
  Increment8(Register8),
  Decrement8(Register8),
  IncrementHLIndirect,
  DecrementHLIndirect,
  Load8(Register8, Register8),
  Load8Immediate(Register8, u8),
  Add8(Register8, Register8),
  AddWithCarry8(Register8, Register8),
  AddAbsolute8(u8),
  AddAbsoluteWithCarry8(u8),
  AddHL(Register16),
  AddIndirect,
  AddIndirectWithCarry,
  Sub8(Register8, Register8),
  SubAbsolute8(u8),
  SubWithCarry8(Register8, Register8),
  SubAbsoluteWithCarry8(u8),
  SubIndirect,
  SubIndirectWithCarry,
  And8(Register8, Register8),
  AndAbsolute8(u8),
  AndIndirect,
  Or8(Register8, Register8),
  OrAbsolute8(u8),
  OrIndirect,
  Xor8(Register8, Register8),
  XorAbsolute8(u8),
  XorIndirect,
  Compare8(Register8),
  CompareAbsolute8(u8),
  CompareIndirect,
  RotateLeftCarryA,
  RotateLeftCarry(Register8),
  RotateLeftCarryIndirect,
  RotateLeftA,
  RotateLeft(Register8),
  RotateLeftIndirect,
  RotateRightCarryA,
  RotateRightCarry(Register8),
  RotateRightCarryIndirect,
  RotateRightA,
  RotateRight(Register8),
  RotateRightIndirect,
  ShiftLeft(Register8),
  ShiftLeftIndirect,
  ShiftRight(Register8),
  ShiftRightIndirect,
  ShiftRightLogical(Register8),
  ShiftRightLogicalIndirect,
  Swap(Register8),
  SwapIndirect,
  BitTest(Register8, u8),
  BitTestIndirect(u8),
  BitClear(Register8, u8),
  BitClearIndirect(u8),
  BitSet(Register8, u8),
  BitSetIndirect(u8),
  LoadStackPointerToMemory(u16),
  AddSP(i8),
  LoadAToMemory(u16),
  LoadAFromMemory(u16),
  LoadToHighMem,
  LoadFromHighMem,
  LoadStackOffset(i8),
  LoadToStackPointer,  
  DAA,
  ComplementA,
  ComplementCarryFlag,
  SetCarryFlag,
  Jump(JumpCondition, u16),
  JumpHL,
  JumpRelative(JumpCondition, i8),
  Call(JumpCondition, u16),
  Return(JumpCondition),
  ResetVector(u16),
  ReturnFromInterrupt,
  InterruptEnable,
  InterruptDisable,
  Push(Register16),
  Pop(Register16),
}

impl Op {
  pub fn is_block_end(&self) -> bool {
    match self {
      Op::Jump(_, _) => true,
      Op::JumpHL => true,
      Op::JumpRelative(_, _) => true,
      Op::Call(_, _) => true,
      Op::ResetVector(_) => true,
      Op::Return(_) => true,
      Op::ReturnFromInterrupt => true,
      Op::InterruptEnable => true,
      Op::InterruptDisable => true,
      Op::Stop => true,
      Op::Halt => true,
      _ => false,
    }
  }
}

impl std::fmt::Display for Op {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Op::Invalid(_) => f.write_str("INVALID"),
      Op::NoOp => f.write_str("NOP"),
      Op::Stop => f.write_str("STOP 0"),
      Op::Halt => f.write_str("HALT"),
      Op::Load16(reg, value) =>
        f.write_fmt(format_args!("LD {}, {:#06X}", reg, value)),
      Op::LoadToIndirect(indirect, reg) =>
        f.write_fmt(format_args!("LD {}, {}", indirect, reg)),
      Op::LoadImmediateToHLIndirect(value) =>
        f.write_fmt(format_args!("LD (HL), {:#04X}", value)),
      Op::LoadFromIndirect(reg, indirect) =>
        f.write_fmt(format_args!("LD {}, {}", reg, indirect)),
      Op::Increment16(reg) => f.write_fmt(format_args!("INC {}", reg)),
      Op::Decrement16(reg) => f.write_fmt(format_args!("DEC {}", reg)),
      Op::Increment8(reg) => f.write_fmt(format_args!("INC {}", reg)),
      Op::Decrement8(reg) => f.write_fmt(format_args!("DEC {}", reg)),
      Op::IncrementHLIndirect => f.write_str("INC (HL)"),
      Op::DecrementHLIndirect => f.write_str("DEC (HL)"),
      Op::Load8(dest, src) =>
        f.write_fmt(format_args!("LD {}, {}", dest, src)),
      Op::Load8Immediate(dest, value) =>
        f.write_fmt(format_args!("LD {}, {:#04X}", dest, value)),
      Op::Add8(dest, src) =>
        f.write_fmt(format_args!("ADD {}, {}", dest, src)),
      Op::AddWithCarry8(dest, src) =>
        f.write_fmt(format_args!("ADC {}, {}", dest, src)),
      Op::AddAbsolute8(value) => f.write_fmt(format_args!("ADD A, {:#04X}", value)),
      Op::AddAbsoluteWithCarry8(value) => f.write_fmt(format_args!("ADC A, {:#04X}", value)),
      Op::AddHL(reg) => f.write_fmt(format_args!("ADD HL, {}", reg)),
      Op::AddIndirect => f.write_str("ADD A, (HL)"),
      Op::AddIndirectWithCarry => f.write_str("ADC A, (HL)"),
      Op::Sub8(dest, src) =>
        f.write_fmt(format_args!("SUB {}, {}", dest, src)),
      Op::SubAbsolute8(value) => f.write_fmt(format_args!("SUB A, {:#04X}", value)),
      Op::SubWithCarry8(dest, src) =>
        f.write_fmt(format_args!("SBC {}, {}", dest, src)),
      Op::SubAbsoluteWithCarry8(value) => f.write_fmt(format_args!("SBC A, {:#04X}", value)),
      Op::SubIndirect => f.write_str("SUB (HL)"),
      Op::SubIndirectWithCarry => f.write_str("SBC (HL)"),
      Op::And8(dest, src) =>
        f.write_fmt(format_args!("AND {}, {}", dest, src)),
      Op::AndAbsolute8(value) => f.write_fmt(format_args!("AND A, {:#04X}", value)),
      Op::AndIndirect => f.write_str("AND (HL)"),
      Op::Or8(dest, src) =>
        f.write_fmt(format_args!("OR {}, {}", dest, src)),
      Op::OrAbsolute8(value) => f.write_fmt(format_args!("OR A, {:#04X}", value)),
      Op::OrIndirect => f.write_str("OR (HL)"),
      Op::Xor8(dest, src) =>
        f.write_fmt(format_args!("XOR {}, {}", dest, src)),
      Op::XorAbsolute8(value) => f.write_fmt(format_args!("XOR A, {:#04X}", value)),
      Op::XorIndirect => f.write_str("XOR (HL)"),
      Op::Compare8(reg) => f.write_fmt(format_args!("CP {}", reg)),
      Op::CompareAbsolute8(value) => f.write_fmt(format_args!("CP A, {:#04X}", value)),
      Op::CompareIndirect => f.write_str("CP (HL)"),
      Op::RotateLeftCarryA => f.write_str("RLCA"),
      Op::RotateLeftCarry(reg) => f.write_fmt(format_args!("RLC {}", reg)),
      Op::RotateLeftCarryIndirect => f.write_str("RLC (HL)"),
      Op::RotateLeftA => f.write_str("RLA"),
      Op::RotateLeft(reg) => f.write_fmt(format_args!("RL {}", reg)),
      Op::RotateLeftIndirect => f.write_str("RL (HL)"),
      Op::RotateRightCarryA => f.write_str("RRCA"),
      Op::RotateRightCarry(reg) => f.write_fmt(format_args!("RRC {}", reg)),
      Op::RotateRightCarryIndirect => f.write_str("RRC (HL)"),
      Op::RotateRightA => f.write_str("RRA"),
      Op::RotateRight(reg) => f.write_fmt(format_args!("RR {}", reg)),
      Op::RotateRightIndirect => f.write_str("RR (HL)"),
      Op::ShiftLeft(reg) => f.write_fmt(format_args!("SLA {}", reg)),
      Op::ShiftLeftIndirect => f.write_str("SLA (HL)"),
      Op::ShiftRight(reg) => f.write_fmt(format_args!("SRA {}", reg)),
      Op::ShiftRightIndirect => f.write_str("SRA (HL)"),
      Op::ShiftRightLogical(reg) => f.write_fmt(format_args!("SRL {}", reg)),
      Op::ShiftRightLogicalIndirect => f.write_str("SRL (HL)"),
      Op::Swap(reg) => f.write_fmt(format_args!("SWAP {}", reg)),
      Op::SwapIndirect => f.write_str("SWAP (HL)"),
      Op::BitTest(reg, bit) => f.write_fmt(format_args!("BIT {:#010b}, {}", bit, reg)),
      Op::BitTestIndirect(bit) => f.write_fmt(format_args!("BIT {:#010b}, (HL)", bit)),
      Op::BitClear(reg, bit) => f.write_fmt(format_args!("RES {:#010b}, {}", bit, reg)),
      Op::BitClearIndirect(bit) => f.write_fmt(format_args!("RES {:#010b}, (HL)", bit)),
      Op::BitSet(reg, bit) => f.write_fmt(format_args!("SET {:#010b}, {}", bit, reg)),
      Op::BitSetIndirect(bit) => f.write_fmt(format_args!("SET {:#010b}, (HL)", bit)),
      Op::LoadStackPointerToMemory(addr) =>
        f.write_fmt(format_args!("LD ({:#06X}), SP", addr)),
      Op::AddSP(offset) => f.write_fmt(format_args!("ADD SP, {:#04X}", offset)),
      Op::LoadAToMemory(addr) => f.write_fmt(format_args!("LD ({:#06X}), A", addr)),
      Op::LoadAFromMemory(addr) => f.write_fmt(format_args!("LD A, ({:#06X})", addr)),
      Op::LoadToHighMem => f.write_str("LD (C), A"),
      Op::LoadFromHighMem => f.write_str("LD A, (C)"),
      Op::LoadStackOffset(off) => f.write_fmt(format_args!("LDHL SP, {:#04X}", off)),
      Op::LoadToStackPointer => f.write_str("LD SP, HL"),
      Op::DAA => f.write_str("DAA"),
      Op::ComplementA => f.write_str("CPL"),
      Op::ComplementCarryFlag => f.write_str("CCF"),
      Op::SetCarryFlag => f.write_str("SCF"),
      Op::Jump(cond, addr) => match cond {
        JumpCondition::Always => f.write_fmt(format_args!("JP {:#06X}", addr)),
        JumpCondition::Carry => f.write_fmt(format_args!("JP C, {:#06X}", addr)),
        JumpCondition::Zero => f.write_fmt(format_args!("JP Z, {:#06X}", addr)),
        JumpCondition::NoCarry => f.write_fmt(format_args!("JP NC, {:#06X}", addr)),
        JumpCondition::NonZero => f.write_fmt(format_args!("JP NZ, {:#06X}", addr)),
      },
      Op::JumpHL => f.write_str("JP (HL)"),
      Op::JumpRelative(cond, offset) => match cond {
        JumpCondition::Always => f.write_fmt(format_args!("JR {:#04X}", offset)),
        JumpCondition::Carry => f.write_fmt(format_args!("JR C, {:#04X}", offset)),
        JumpCondition::Zero => f.write_fmt(format_args!("JR Z, {:#04X}", offset)),
        JumpCondition::NoCarry => f.write_fmt(format_args!("JR NC, {:#04X}", offset)),
        JumpCondition::NonZero => f.write_fmt(format_args!("JR NZ, {:#04X}", offset)),
      },
      Op::Call(cond, addr) => match cond {
        JumpCondition::Always => f.write_fmt(format_args!("CALL {:#06X}", addr)),
        JumpCondition::Carry => f.write_fmt(format_args!("CALL C, {:#06X}", addr)),
        JumpCondition::Zero => f.write_fmt(format_args!("CALL Z, {:#06X}", addr)),
        JumpCondition::NoCarry => f.write_fmt(format_args!("CALL NC, {:#06X}", addr)),
        JumpCondition::NonZero => f.write_fmt(format_args!("CALL NZ, {:#06X}", addr)),
      },
      Op::Return(cond) => match cond {
        JumpCondition::Always => f.write_str("RET"),
        JumpCondition::Carry => f.write_str("RET C"),
        JumpCondition::Zero => f.write_str("RET Z"),
        JumpCondition::NoCarry => f.write_str("RET NC"),
        JumpCondition::NonZero => f.write_str("RET NZ"),
      },
      Op::ResetVector(vec) => f.write_fmt(format_args!("RST {:#04X}", vec)),
      Op::ReturnFromInterrupt => f.write_str("RETI"),
      Op::InterruptEnable => f.write_str("EI"),
      Op::InterruptDisable => f.write_str("DI"),
      Op::Push(reg) => f.write_fmt(format_args!("PUSH {}", reg)),
      Op::Pop(reg) => f.write_fmt(format_args!("POP {}", reg)),
    }
  }
}

#[derive(Copy, Clone)]
pub enum IndirectLocation {
  BC,
  DE,
  HL,
  HLIncrement,
  HLDecrement,
}

impl std::fmt::Display for IndirectLocation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      IndirectLocation::BC => f.write_str("(BC)"),
      IndirectLocation::DE => f.write_str("(DE)"),
      IndirectLocation::HL => f.write_str("(HL)"),
      IndirectLocation::HLIncrement => f.write_str("(HL+)"),
      IndirectLocation::HLDecrement => f.write_str("(HL-)"),
    }
  }
}

#[derive(Copy, Clone)]
pub enum Register8 {
  A,
  B,
  C,
  D,
  E,
  H,
  L,
}

impl std::fmt::Display for Register8 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Register8::A => f.write_str("A"),
      Register8::B => f.write_str("B"),
      Register8::C => f.write_str("C"),
      Register8::D => f.write_str("D"),
      Register8::E => f.write_str("E"),
      Register8::H => f.write_str("H"),
      Register8::L => f.write_str("L"),
    }
  }
}

#[derive(Copy, Clone)]
pub enum Register16 {
  AF,
  BC,
  DE,
  HL,
  SP,
}

impl std::fmt::Display for Register16 {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Register16::AF => f.write_str("AF"),
      Register16::BC => f.write_str("BC"),
      Register16::DE => f.write_str("DE"),
      Register16::HL => f.write_str("HL"),
      Register16::SP => f.write_str("SP"),
    }
  }
}

pub enum JumpCondition {
  Always,
  Zero,
  NonZero,
  Carry,
  NoCarry,
}

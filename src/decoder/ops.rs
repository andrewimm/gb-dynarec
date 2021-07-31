pub enum Op {
  Invalid(u8),
  NoOp,
  Stop,
  Halt,
  Load16(Register16, u16),
  LoadToIndirect(IndirectLocation, Source8),
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
  RotateLeftA,
  RotateLeft(Register8),
  RotateRightCarryA,
  RotateRightCarry(Register8),
  RotateRightA,
  RotateRight(Register8),
  ShiftLeft(Register8),
  ShiftLeftIndirect,
  ShiftRight(Register8),
  ShiftRightIndirect,
  ShiftRightLogical(Register8),
  ShiftRightLogicalIndirect,
  Swap(Register8),
  BitTest(Register8, u8),
  BitClear(Register8, u8),
  BitSet(Register8, u8),
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

      Op::Load8Immediate(reg, value) => f.write_fmt(format_args!("LD {}, {:#04X}", reg, value)),
      
      Op::LoadAToMemory(addr) => f.write_fmt(format_args!("LD ({:#06X}), A", addr)),
      Op::LoadAFromMemory(addr) => f.write_fmt(format_args!("LD A, ({:#06X})", addr)),
      Op::LoadToHighMem => f.write_str("LD (C), A"),
      Op::LoadFromHighMem => f.write_str("LD A, (C)"),
      Op::LoadStackOffset(off) => f.write_fmt(format_args!("LD HL, SP + {:#04X}", off)),
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
      _ => f.write_str("???"),
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

#[derive(Copy, Clone)]
pub enum Source8 {
  A,
  B,
  C,
  D,
  E,
  H,
  L,
  Literal(u8),
}

#[derive(Copy, Clone)]
pub enum Register8 {
  A,
  F,
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
      Register8::F => f.write_str("F"),
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

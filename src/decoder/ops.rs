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

#[derive(Copy, Clone)]
pub enum Register16 {
  AF,
  BC,
  DE,
  HL,
  SP,
}

pub enum JumpCondition {
  Always,
  Zero,
  NonZero,
  Carry,
  NoCarry,
}

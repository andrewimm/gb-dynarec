use crate::decoder::decode;
use crate::decoder::ops::{Op, Register8, Register16, IndirectLocation, JumpCondition};
use crate::cpu::{Registers, self};
use crate::mem::{get_executable_memory_slice, memory_read_byte, memory_write_byte, memory_write_word, MemoryAreas};

pub fn run_code_block(registers: &mut Registers, mem: *mut MemoryAreas) -> u8 {
  let mut status = cpu::STATUS_NORMAL;
  loop {
    match run_next_op(registers, mem) {
      Some((op_status, should_break)) => {
        status = op_status;
        if should_break {
          break;
        }
      },
      None => break,
    }
    /*
    let index = registers.ip as usize;
    let code_slice = get_executable_memory_slice(index, mem);
    if code_slice.len() < 1 {
      break;
    }
    let (next_op, length, cycles) = decode(code_slice);
    let should_break = next_op.is_block_end();
    status = run_op(next_op, registers, mem, length as u32);
    
    registers.cycles += (cycles / 4) as u32;
    
    if should_break {
      break;
    }
    */
  }

  status
}

pub fn run_next_op(registers: &mut Registers, mem: *mut MemoryAreas) -> Option<(u8, bool)> {
  let index = registers.ip as usize;
  let code_slice = get_executable_memory_slice(index, mem);
  if code_slice.len() < 1 {
    return None;
  }
  let (next_op, length, cycles) = decode(code_slice);
  let should_break = next_op.is_block_end();
  let status = run_op(next_op, registers, mem, length as u32);
  registers.cycles += (cycles / 4) as u32;

  return Some((status, should_break));
}

pub fn run_op(op: Op, registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  match op {
    Op::NoOp => {
      registers.ip += length;
      cpu::STATUS_NORMAL
    },
    Op::Load8(dest, src) => interp_load_8_register(dest, src, registers, length), 
    Op::Load16(dest, value) => interp_load_16(dest, value, registers, length),
    Op::LoadToIndirect(location, value) => interp_load_to_indirect(location, value, registers, mem, length),
    Op::LoadImmediateToHLIndirect(value) => interp_load_immediate_to_hl_indirect(value, registers, mem, length),
    Op::LoadFromIndirect(reg, location) => interp_load_from_indirect(reg, location, registers, mem, length),
    Op::Load8Immediate(reg, value) => interp_load_8_immediate(reg, value, registers, length),
    Op::Increment8(reg) => interp_increment_8(reg, registers, length),
    Op::Decrement8(reg) => interp_decrement_8(reg, registers, length),
    Op::Increment16(reg) => interp_increment_16(reg, registers, length), 
    Op::Decrement16(reg) => interp_decrement_16(reg, registers, length),
    Op::IncrementHLIndirect => interp_increment_hl_indirect(registers, mem, length),
    Op::DecrementHLIndirect => interp_decrement_hl_indirect(registers, mem, length),
    Op::Add8(dest, src) => interp_add_register_8(dest, src, registers, length),
    Op::AddWithCarry8(dest, src) => interp_add_register_8_with_carry(dest, src, registers, length),
    Op::AddHL(src) => interp_add_hl(src, registers, length),
    Op::AddAbsolute8(value) => interp_add_absolute_8(value, registers, length),
    Op::AddAbsoluteWithCarry8(value) => interp_add_absolute_8_with_carry(value, registers, length),
    Op::AddIndirect => interp_add_indirect(registers, mem, length),
    Op::AddIndirectWithCarry => interp_add_indirect_with_carry(registers, mem, length),
    Op::Sub8(dest, src) => interp_sub_register_8(dest, src, registers, length),
    Op::SubWithCarry8(dest, src) => interp_sub_register_8_with_carry(dest, src, registers, length),
    Op::SubAbsolute8(value) => interp_sub_absolute_8(value, registers, length), 
    Op::SubAbsoluteWithCarry8(value) => interp_sub_absolute_8_with_carry(value, registers, length),
    Op::SubIndirect => interp_sub_indirect(registers, mem, length),
    Op::SubIndirectWithCarry => interp_sub_indirect_with_carry(registers, mem, length),
    Op::And8(dest, src) => interp_and_register_8(dest, src, registers, length),
    Op::AndAbsolute8(value) => interp_and_absolute_8(value, registers, length),
    Op::AndIndirect => interp_and_indirect(registers, mem, length),
    Op::Xor8(dest, src) => interp_xor_register_8(dest, src, registers, length), 
    Op::XorAbsolute8(value) => interp_xor_absolute_8(value, registers, length),
    Op::XorIndirect => interp_xor_indirect(registers, mem, length),
    Op::Or8(dest, src) => interp_or_register_8(dest, src, registers, length), 
    Op::OrAbsolute8(value) => interp_or_absolute_8(value, registers, length),
    Op::OrIndirect => interp_or_indirect(registers, mem, length),
    Op::Compare8(reg) => interp_compare_register_8(reg, registers, length), 
    Op::CompareAbsolute8(value) => interp_compare_absolute_8(value, registers, length),
    Op::CompareIndirect => interp_compare_indirect(registers, mem, length),
    Op::RotateLeftA => interp_rotate_left_a(registers, length),
    Op::RotateLeftCarryA => interp_rotate_left_carry_a(registers, length),
    Op::RotateLeft(reg) => interp_rotate_left(reg, registers, length),
    Op::RotateLeftIndirect => interp_rotate_left_indirect(registers, mem, length),
    Op::RotateLeftCarry(reg) => interp_rotate_left_carry(reg, registers, length),
    Op::RotateLeftCarryIndirect => interp_rotate_left_carry_indirect(registers, mem, length),
    Op::RotateRightA => interp_rotate_right_a(registers, length),
    Op::RotateRightCarryA => interp_rotate_right_carry_a(registers, length),
    Op::RotateRight(reg) => interp_rotate_right(reg, registers, length),
    Op::RotateRightIndirect => interp_rotate_right_indirect(registers, mem, length),
    Op::RotateRightCarry(reg) => interp_rotate_right_carry(reg, registers, length),
    Op::RotateRightCarryIndirect => interp_rotate_right_carry_indirect(registers, mem, length),
    Op::ShiftLeft(reg) => interp_shift_left(reg, registers, length),
    Op::ShiftLeftIndirect => interp_shift_left_indirect(registers, mem, length),
    Op::ShiftRight(reg) => interp_shift_right(reg, registers, length),
    Op::ShiftRightIndirect => interp_shift_right_indirect(registers, mem, length),
    Op::ShiftRightLogical(reg) => interp_shift_right_logical(reg, registers, length),
    Op::ShiftRightLogicalIndirect => interp_shift_right_logical_indirect(registers, mem, length),
    Op::ComplementA => interp_complement_a(registers, length),
    Op::SetCarryFlag => interp_set_carry(registers, length),
    Op::ComplementCarryFlag => interp_complement_carry(registers, length),
    Op::BitSet(reg, mask) => interp_bit_set(reg, mask, registers, length),
    Op::BitSetIndirect(mask) => interp_bit_set_indirect(mask, registers, mem, length),
    Op::BitClear(reg, mask) => interp_bit_clear(reg, mask, registers, length),
    Op::BitClearIndirect(mask) => interp_bit_clear_indirect(mask, registers, mem, length),
    Op::BitTest(reg, mask) => interp_bit_test(reg, mask, registers, length),
    Op::BitTestIndirect(mask) => interp_bit_test_indirect(mask, registers, mem, length),
    Op::Swap(reg) => interp_swap(reg, registers, length),
    Op::SwapIndirect => interp_swap_indirect(registers, mem, length),
    Op::LoadStackPointerToMemory(addr) => interp_load_sp_to_memory(addr, registers, mem, length),
    Op::LoadAToMemory(addr, _) => interp_load_a_to_memory(addr, registers, mem, length),
    Op::LoadAFromMemory(addr, _) => interp_load_a_from_memory(addr, registers, mem, length),
    Op::LoadToHighMem => interp_load_to_himem(registers, mem, length),
    Op::LoadFromHighMem => interp_load_from_himem(registers, mem, length),
    Op::Push(reg) => interp_push(reg, registers, mem, length),
    Op::Pop(reg) => interp_pop(reg, registers, mem, length),
    Op::AddSP(offset) => interp_add_sp(offset, registers, length),
    Op::LoadToStackPointer => interp_load_to_sp(registers, length),
    Op::LoadStackOffset(offset) => interp_load_stack_offset(offset, registers, length),
    Op::DAA => interp_daa(registers, length),

    Op::Jump(cond, address) => interp_jump(cond, address, registers), 
    Op::JumpHL => interp_jump_hl(registers),
    Op::JumpRelative(cond, offset) => interp_jump_relative(cond, offset, registers),
    Op::Call(cond, address) => interp_call(cond, address, registers, mem),
    Op::ResetVector(vector) => interp_reset(vector, registers, mem),
    Op::Return(cond) => interp_return(cond, registers, mem),
    Op::ReturnFromInterrupt => interp_reti(registers, mem),

    Op::Stop => {
      registers.ip += 1;
      cpu::STATUS_STOP
    },
    Op::Halt => {
      registers.ip += 1;
      cpu::STATUS_HALT
      },
    Op::InterruptEnable => {
      registers.ip += 1;
      cpu::STATUS_INTERRUPT_ENABLE
    },
    Op::InterruptDisable => {
      registers.ip += 1;
      cpu::STATUS_INTERRUPT_DISABLE
    },

    Op::Invalid(code) => panic!("Invalid OP: {:#04x}", code),
  }
}

// Helpers for common actions

fn get_register(registers: &Registers, reg: Register8) -> u8 {
  match reg {
    Register8::A => registers.get_a(),
    Register8::B => registers.get_b(),
    Register8::C => registers.get_c(),
    Register8::D => registers.get_d(),
    Register8::E => registers.get_e(),
    Register8::H => registers.get_h(),
    Register8::L => registers.get_l(),
  }
}

fn get_register_16(registers: &Registers, reg: Register16) -> u16 {
  (match reg {
    Register16::AF => registers.af,
    Register16::BC => registers.bc,
    Register16::DE => registers.de,
    Register16::HL => registers.hl,
    Register16::SP => registers.sp,
  }) as u16
}

fn set_register(registers: &mut Registers, reg: Register8, value: u8) {
  match reg {
    Register8::A => registers.set_a(value),
    Register8::B => registers.set_b(value),
    Register8::C => registers.set_c(value),
    Register8::D => registers.set_d(value),
    Register8::E => registers.set_e(value),
    Register8::H => registers.set_h(value),
    Register8::L => registers.set_l(value),
  }
}

#[inline(always)]
fn set_register_16(registers: &mut Registers, reg: Register16, value: u16) {
  match reg {
    Register16::AF => registers.af = value as u32,
    Register16::BC => registers.bc = value as u32,
    Register16::DE => registers.de = value as u32,
    Register16::HL => registers.hl = value as u32,
    Register16::SP => registers.sp = value as u32,
  }
}

#[inline(always)]
fn map_indirect_to_register(location: IndirectLocation) -> Register16 {
  match location {
    IndirectLocation::BC => Register16::BC,
    IndirectLocation::DE => Register16::DE,
    IndirectLocation::HL
      | IndirectLocation::HLIncrement
      | IndirectLocation::HLDecrement => Register16::HL,
  }
}

#[inline(always)]
fn apply_mask(registers: &mut Registers, mask: u8) {
  // clear only the masked bits
  let mask16 = 0xff00 | !(mask as u16);
  registers.af &= mask16 as u32;
}

#[inline(always)]
fn test_zero(registers: &mut Registers, value: u8) {
  if value == 0 {
    registers.af |= 0x80;
  }
}

#[inline(always)]
fn test_half_carry(registers: &mut Registers, flag: bool) {
  if flag {
    registers.af |= 0x20;
  }
}

#[inline(always)]
fn set_half_carry(registers: &mut Registers) {
  registers.af |= 0x20;
}

#[inline(always)]
fn test_carry(registers: &mut Registers, flag: bool) {
  if flag {
    registers.af |= 0x10;
  }
}

#[inline(always)]
fn set_carry(registers: &mut Registers) {
  registers.af |= 0x10;
}

#[inline(always)]
fn complement_carry(registers: &mut Registers) {
  registers.af ^= 0x10;
}

#[inline(always)]
fn set_negative(registers: &mut Registers) {
  registers.af |= 0x40;
}

#[inline(always)]
fn carry_add(a: u8, b: u8) -> (u8, bool, bool) {
  let (value, overflow) = a.overflowing_add(b);
  let hc = {
    (a & 0x0f).wrapping_add(b & 0x0f) & 0x10 != 0
  };
  (value, overflow, hc)
}

#[inline(always)]
fn carry_adc(a: u8, b: u8, af: u32) -> (u8, bool, bool) {
  let extra = if af & 0x10 != 0 { 1 } else { 0 };
  let (value, overflow) = {
    let (partial_value, partial_overflow) = a.overflowing_add(b);
    let (final_value, final_overflow) = partial_value.overflowing_add(extra);
    (final_value, partial_overflow || final_overflow)
  };
  let hc = {
    (a & 0x0f).wrapping_add(b & 0x0f).wrapping_add(extra) & 0x10 != 0
  };
  (value, overflow, hc)
}

#[inline(always)]
fn carry_sub(a: u8, b: u8) -> (u8, bool, bool) {
  let (value, overflow) = a.overflowing_sub(b);
  let hc = {
    (a & 0x0f).wrapping_sub(b & 0x0f) & 0x10 != 0
  };
  (value, overflow, hc)
}

#[inline(always)]
fn carry_sbc(a: u8, b: u8, af: u32) -> (u8, bool, bool) {
  let extra = if af & 0x10 != 0 { 1 } else { 0 };
  let (value, overflow) = {
    let (partial_value, partial_overflow) = a.overflowing_sub(b);
    let (final_value, final_overflow) = partial_value.overflowing_sub(extra);
    (final_value, partial_overflow || final_overflow)
  };
  let hc = {
    (a & 0x0f).wrapping_sub(b & 0x0f).wrapping_sub(extra) & 0x10 != 0
  };
  (value, overflow, hc)
}

#[inline(always)]
fn carry_add_16(a: u16, b: u16) -> (u16, bool, bool) {
  let (value, overflow) = a.overflowing_add(b);
  let hc = {
    (a & 0x0fff).wrapping_add(b & 0x0fff) & 0x1000 != 0
  };
  (value, overflow, hc)
}

#[inline(always)]
fn push(value: u16, registers: &mut Registers, mem: *mut MemoryAreas) {
  let mut addr = get_register_16(registers, Register16::SP).wrapping_sub(1);
  memory_write_byte(mem, addr, (value >> 8) as u8);
  addr = addr.wrapping_sub(1);
  memory_write_byte(mem, addr, (value & 0xff) as u8);
  set_register_16(registers, Register16::SP, addr);
}

#[inline(always)]
fn pop(registers: &mut Registers, mem: *mut MemoryAreas) -> u16 {
  let mut addr = get_register_16(registers, Register16::SP);
  let low = memory_read_byte(mem, addr) as u16;
  addr = addr.wrapping_add(1);
  let high = (memory_read_byte(mem, addr) as u16) << 8;
  addr = addr.wrapping_add(1);
  set_register_16(registers, Register16::SP, addr);
  high | low
}

fn rotate_left_through_carry(reg: Register8, registers: &mut Registers) -> (u8, bool) {
  let prev_carry = ((registers.af & 0x10) >> 4) as u8;
  let value = get_register(registers, reg);
  let new_carry = value & 0x80 == 0x80;
  let result = (value << 1) | prev_carry;
  set_register(registers, reg, result);
  (result, new_carry)
}

fn rotate_left(reg: Register8, registers: &mut Registers) -> (u8, bool) {
  let value = get_register(registers, reg);
  let carry_bit = (value & 0x80) >> 7;
  let has_carry = carry_bit != 0;
  let result = (value << 1) | carry_bit;
  set_register(registers, reg, result);
  (result, has_carry)
}

fn rotate_right_through_carry(reg: Register8, registers: &mut Registers) -> (u8, bool) {
  let prev_carry = ((registers.af & 0x10) << 3) as u8;
  let value = get_register(registers, reg);
  let new_carry = value & 0x01 == 0x01;
  let result = (value >> 1) | prev_carry;
  set_register(registers, reg, result);
  (result, new_carry)
}

fn rotate_right(reg: Register8, registers: &mut Registers) -> (u8, bool) {
  let value = get_register(registers, reg);
  let carry_bit = (value & 0x01) << 7;
  let has_carry = carry_bit != 0;
  let result = (value >> 1) | carry_bit;
  set_register(registers, reg, result);
  (result, has_carry)
}

// Implementations of CPU operations

fn interp_load_8_register(dest: Register8, src: Register8, registers: &mut Registers, length: u32) -> u8 {
  let value = get_register(registers, src);
  set_register(registers, dest, value);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_load_16(dest: Register16, value: u16, registers: &mut Registers, length: u32) -> u8 {
  set_register_16(registers, dest, value);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_load_to_indirect(location: IndirectLocation, reg: Register8, registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let value = get_register(registers, reg);
  let address_register = map_indirect_to_register(location);
  let address = get_register_16(registers, address_register);
  memory_write_byte(mem, address, value);
  match location {
    IndirectLocation::HLIncrement => registers.hl = registers.hl.wrapping_add(1),
    IndirectLocation::HLDecrement => registers.hl = registers.hl.wrapping_sub(1),
    _ => (),
  }
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_load_8_immediate(reg: Register8, value: u8, registers: &mut Registers, length: u32) -> u8 {
  set_register(registers, reg, value);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_load_immediate_to_hl_indirect(value: u8, registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let address = get_register_16(registers, Register16::HL);
  memory_write_byte(mem, address, value);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_load_from_indirect(reg: Register8, location: IndirectLocation, registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let address_register = map_indirect_to_register(location);
  let address = get_register_16(registers, address_register);
  let value = memory_read_byte(mem, address);
  set_register(registers, reg, value);
  match location {
    IndirectLocation::HLIncrement => registers.hl = registers.hl.wrapping_add(1),
    IndirectLocation::HLDecrement => registers.hl = registers.hl.wrapping_sub(1),
    _ => (),
  }
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_increment_8(reg: Register8, registers: &mut Registers, length: u32) -> u8 {
  let value = get_register(registers, reg);
  let (incremented, _, half_carry) = carry_add(value, 1);
  set_register(registers, reg, incremented);
  apply_mask(registers, 0xe0);
  test_half_carry(registers, half_carry);
  test_zero(registers, incremented);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_decrement_8(reg: Register8, registers: &mut Registers, length: u32) -> u8 {
  let value = get_register(registers, reg);
  let (decremented, _, half_carry) = carry_sub(value, 1);
  set_register(registers, reg, decremented);
  apply_mask(registers, 0xe0);
  test_half_carry(registers, half_carry);
  set_negative(registers);
  test_zero(registers, decremented);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_increment_16(reg: Register16, registers: &mut Registers, length: u32) -> u8 {
  let value = get_register_16(registers, reg);
  set_register_16(registers, reg, value.wrapping_add(1));
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_decrement_16(reg: Register16, registers: &mut Registers, length: u32) -> u8 {
  let value = get_register_16(registers, reg);
  set_register_16(registers, reg, value.wrapping_sub(1));
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_increment_hl_indirect(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let address = registers.hl as u16;
  let value = memory_read_byte(mem, address);
  let (incremented, _, half_carry) = carry_add(value, 1);
  memory_write_byte(mem, address, incremented);
  apply_mask(registers, 0xe0);
  test_half_carry(registers, half_carry);
  test_zero(registers, incremented);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_decrement_hl_indirect(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let address = registers.hl as u16;
  let value = memory_read_byte(mem, address);
  let (decremented, _, half_carry) = carry_sub(value, 1);
  memory_write_byte(mem, address, decremented);
  apply_mask(registers, 0xe0);
  test_half_carry(registers, half_carry);
  set_negative(registers);
  test_zero(registers, decremented);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_add_register_8(dest: Register8, src: Register8, registers: &mut Registers, length: u32) -> u8 {
  let orig = get_register(registers, dest);
  let to_add = get_register(registers, src);
  let (sum, carry, half_carry) = carry_add(orig, to_add);
  set_register(registers, dest, sum);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_half_carry(registers, half_carry);
  test_zero(registers, sum);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_add_register_8_with_carry(dest: Register8, src: Register8, registers: &mut Registers, length: u32) -> u8 {
  let orig = get_register(registers, dest);
  let to_add = get_register(registers, src);
  let (sum, carry, half_carry) = carry_adc(orig, to_add, registers.af);
  set_register(registers, dest, sum);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_half_carry(registers, half_carry);
  test_zero(registers, sum);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_add_absolute_8(value: u8, registers: &mut Registers, length: u32) -> u8 {
  let orig = get_register(registers, Register8::A);
  let (sum, carry, half_carry) = carry_add(orig, value);
  set_register(registers, Register8::A, sum);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_half_carry(registers, half_carry);
  test_zero(registers, sum);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_add_absolute_8_with_carry(value: u8, registers: &mut Registers, length: u32) -> u8 {
  let orig = get_register(registers, Register8::A);
  let (sum, carry, half_carry) = carry_adc(orig, value, registers.af);
  set_register(registers, Register8::A, sum);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_half_carry(registers, half_carry);
  test_zero(registers, sum);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_add_hl(src: Register16, registers: &mut Registers, length: u32) -> u8 {
  let orig = get_register_16(registers, Register16::HL);
  let to_add = get_register_16(registers, src);
  let (sum, carry, half_carry) = carry_add_16(orig, to_add);
  set_register_16(registers, Register16::HL, sum);
  apply_mask(registers, 0x70);
  test_carry(registers, carry);
  test_half_carry(registers, half_carry);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_add_indirect(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let orig = get_register(registers, Register8::A);
  let address = get_register_16(registers, Register16::HL);
  let to_add = memory_read_byte(mem, address);
  let (sum, carry, half_carry) = carry_add(orig, to_add);
  set_register(registers, Register8::A, sum);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_half_carry(registers, half_carry);
  test_zero(registers, sum);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_add_indirect_with_carry(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let orig = get_register(registers, Register8::A);
  let address = get_register_16(registers, Register16::HL);
  let to_add = memory_read_byte(mem, address);
  let (sum, carry, half_carry) = carry_adc(orig, to_add, registers.af);
  set_register(registers, Register8::A, sum);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_half_carry(registers, half_carry);
  test_zero(registers, sum);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_sub_register_8(dest: Register8, src: Register8, registers: &mut Registers, length: u32) -> u8 {
  let orig = get_register(registers, dest);
  let to_sub = get_register(registers, src);
  let (diff, carry, half_carry) = carry_sub(orig, to_sub);
  set_register(registers, dest, diff);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_half_carry(registers, half_carry);
  set_negative(registers);
  test_zero(registers, diff);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_sub_register_8_with_carry(dest: Register8, src: Register8, registers: &mut Registers, length: u32) -> u8 {
  let orig = get_register(registers, dest);
  let to_sub = get_register(registers, src);
  let (diff, carry, half_carry) = carry_sbc(orig, to_sub, registers.af);
  set_register(registers, dest, diff);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_half_carry(registers, half_carry);
  set_negative(registers);
  test_zero(registers, diff);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_sub_absolute_8(value: u8, registers: &mut Registers, length: u32) -> u8 {
  let orig = get_register(registers, Register8::A);
  let (diff, carry, half_carry) = carry_sub(orig, value);
  set_register(registers, Register8::A, diff);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_half_carry(registers, half_carry);
  set_negative(registers);
  test_zero(registers, diff);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_sub_absolute_8_with_carry(value: u8, registers: &mut Registers, length: u32) -> u8 {
  let orig = get_register(registers, Register8::A);
  let (diff, carry, half_carry) = carry_sbc(orig, value, registers.af);
  set_register(registers, Register8::A, diff);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_half_carry(registers, half_carry);
  set_negative(registers);
  test_zero(registers, diff);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_sub_indirect(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let orig = get_register(registers, Register8::A);
  let address = get_register_16(registers, Register16::HL);
  let to_sub = memory_read_byte(mem, address);
  let (diff, carry, half_carry) = carry_sub(orig, to_sub);
  set_register(registers, Register8::A, diff);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_half_carry(registers, half_carry);
  set_negative(registers);
  test_zero(registers, diff);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_sub_indirect_with_carry(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let orig = get_register(registers, Register8::A);
  let address = get_register_16(registers, Register16::HL);
  let to_sub = memory_read_byte(mem, address);
  let (diff, carry, half_carry) = carry_sbc(orig, to_sub, registers.af);
  set_register(registers, Register8::A, diff);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_half_carry(registers, half_carry);
  set_negative(registers);
  test_zero(registers, diff);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_and_register_8(dest: Register8, src: Register8, registers: &mut Registers, length: u32) -> u8 {
  let orig = get_register(registers, dest);
  let to_and = get_register(registers, src);
  let value = orig & to_and;
  set_register(registers, dest, value);
  apply_mask(registers, 0xf0);
  set_half_carry(registers);
  test_zero(registers, value);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_and_absolute_8(value: u8, registers: &mut Registers, length: u32) -> u8 {
  let orig = get_register(registers, Register8::A);
  let result = orig & value;
  set_register(registers, Register8::A, result);
  apply_mask(registers, 0xf0);
  set_half_carry(registers);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_and_indirect(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let orig = get_register(registers, Register8::A);
  let address = get_register_16(registers, Register16::HL);
  let to_and = memory_read_byte(mem, address);
  let result = orig & to_and;
  set_register(registers, Register8::A, result);
  apply_mask(registers, 0xf0);
  set_half_carry(registers);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_xor_register_8(dest: Register8, src: Register8, registers: &mut Registers, length: u32) -> u8 {
  let orig = get_register(registers, dest);
  let to_xor = get_register(registers, src);
  let value = orig ^ to_xor;
  set_register(registers, dest, value);
  apply_mask(registers, 0xf0);
  test_zero(registers, value);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_xor_absolute_8(value: u8, registers: &mut Registers, length: u32) -> u8 {
  let orig = get_register(registers, Register8::A);
  let result = orig ^ value;
  set_register(registers, Register8::A, result);
  apply_mask(registers, 0xf0);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_xor_indirect(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let orig = get_register(registers, Register8::A);
  let address = get_register_16(registers, Register16::HL);
  let to_xor = memory_read_byte(mem, address);
  let result = orig ^ to_xor;
  set_register(registers, Register8::A, result);
  apply_mask(registers, 0xf0);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_or_register_8(dest: Register8, src: Register8, registers: &mut Registers, length: u32) -> u8 {
  let orig = get_register(registers, dest);
  let to_or = get_register(registers, src);
  let value = orig | to_or;
  set_register(registers, dest, value);
  apply_mask(registers, 0xf0);
  test_zero(registers, value);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_or_absolute_8(value: u8, registers: &mut Registers, length: u32) -> u8 {
  let orig = get_register(registers, Register8::A);
  let result = orig | value;
  set_register(registers, Register8::A, result);
  apply_mask(registers, 0xf0);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_or_indirect(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let orig = get_register(registers, Register8::A);
  let address = get_register_16(registers, Register16::HL);
  let to_or = memory_read_byte(mem, address);
  let result = orig | to_or;
  set_register(registers, Register8::A, result);
  apply_mask(registers, 0xf0);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_compare_register_8(reg: Register8, registers: &mut Registers, length: u32) -> u8 {
  let orig = get_register(registers, Register8::A);
  let to_sub = get_register(registers, reg);
  let (diff, carry, half_carry) = carry_sub(orig, to_sub);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_half_carry(registers, half_carry);
  set_negative(registers);
  test_zero(registers, diff);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_compare_absolute_8(value: u8, registers: &mut Registers, length: u32) -> u8 {
  let orig = get_register(registers, Register8::A);
  let (diff, carry, half_carry) = carry_sub(orig, value);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_half_carry(registers, half_carry);
  set_negative(registers);
  test_zero(registers, diff);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_compare_indirect(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let orig = get_register(registers, Register8::A);
  let address = get_register_16(registers, Register16::HL);
  let value = memory_read_byte(mem, address);
  let (diff, carry, half_carry) = carry_sub(orig, value);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_half_carry(registers, half_carry);
  set_negative(registers);
  test_zero(registers, diff);
  registers.ip += length;
  cpu::STATUS_NORMAL
}


fn interp_complement_a(registers: &mut Registers, length: u32) -> u8 {
  let orig = get_register(registers, Register8::A);
  set_register(registers, Register8::A, !orig);
  set_negative(registers);
  set_half_carry(registers);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_set_carry(registers: &mut Registers, length: u32) -> u8 {
  apply_mask(registers, 0x70);
  set_carry(registers);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_complement_carry(registers: &mut Registers, length: u32) -> u8 {
  apply_mask(registers, 0x60);
  complement_carry(registers);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_bit_set(reg: Register8, mask: u8, registers: &mut Registers, length: u32) -> u8 {
  let value = get_register(registers, reg);
  let result = value | mask;
  set_register(registers, reg, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_bit_set_indirect(mask: u8, registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let address = get_register_16(registers, Register16::HL);
  let value = memory_read_byte(mem, address);
  let result = value | mask;
  memory_write_byte(mem, address, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_bit_clear(reg: Register8, mask: u8, registers: &mut Registers, length: u32) -> u8 {
  let value = get_register(registers, reg);
  let result = value & !mask;
  set_register(registers, reg, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_bit_clear_indirect(mask: u8, registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let address = get_register_16(registers, Register16::HL);
  let value = memory_read_byte(mem, address);
  let result = value & !mask;
  memory_write_byte(mem, address, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_bit_test(reg: Register8, mask: u8, registers: &mut Registers, length: u32) -> u8 {
  let value = get_register(registers, reg);
  let result = value & mask;
  apply_mask(registers, 0xe0);
  set_half_carry(registers);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_bit_test_indirect(mask: u8, registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let address = get_register_16(registers, Register16::HL);
  let value = memory_read_byte(mem, address);
  let result = value & mask;
  apply_mask(registers, 0xe0);
  set_half_carry(registers);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_swap(reg: Register8, registers: &mut Registers, length: u32) -> u8 {
  let value = get_register(registers, reg);
  let high = value >> 4;
  let low = (value & 0x0f) << 4;
  let result = high | low;
  set_register(registers, reg, result);
  apply_mask(registers, 0xf0);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_swap_indirect(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let address = get_register_16(registers, Register16::HL);
  let value = memory_read_byte(mem, address);
  let high = value >> 4;
  let low = (value & 0x0f) << 4;
  let result = high | low;
  memory_write_byte(mem, address, result);
  apply_mask(registers, 0xf0);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_load_sp_to_memory(addr: u16, registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let value = get_register_16(registers, Register16::SP);
  memory_write_word(mem, addr, value);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_load_a_to_memory(addr: u16, registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let value = get_register(registers, Register8::A);
  memory_write_byte(mem, addr, value);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_load_a_from_memory(addr: u16, registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let value = memory_read_byte(mem, addr);
  set_register(registers, Register8::A, value);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_load_to_himem(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let addr = 0xff00 | (get_register(registers, Register8::C) as u16);
  let value = get_register(registers, Register8::A);
  memory_write_byte(mem, addr, value);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_load_from_himem(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let addr = 0xff00 | (get_register(registers, Register8::C) as u16);
  let value = memory_read_byte(mem, addr);
  set_register(registers, Register8::A, value);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_push(reg: Register16, registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let value = get_register_16(registers, reg);
  push(value, registers, mem);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_pop(reg: Register16, registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let mut value = pop(registers, mem);
  if let Register16::AF = reg {
    value &= 0xfff0;
  }

  set_register_16(registers, reg, value);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_add_sp(offset: i8, registers: &mut Registers, length: u32) -> u8 {
  let value = get_register_16(registers, Register16::SP);
  let result = if (offset as u8) & 0x80 == 0 {
    let to_add = offset as u16;
    value.wrapping_add(to_add)
  } else {
    let to_sub = (!(offset as u8) as u16).wrapping_add(1);
    value.wrapping_sub(to_sub)
  };
  let carry = {
    (value & 0xff) + (((offset as u8) as u16) & 0xff) & 0x100 != 0
  };
  let hc = {
    ((value as u8) & 0x0f) + ((offset as u8) & 0x0f) & 0x10 != 0
  };

  set_register_16(registers, Register16::SP, result);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_half_carry(registers, hc);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_load_to_sp(registers: &mut Registers, length: u32) -> u8 {
  let value = get_register_16(registers, Register16::HL);
  set_register_16(registers, Register16::SP, value);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_load_stack_offset(offset: i8, registers: &mut Registers, length: u32) -> u8 {
  let value = get_register_16(registers, Register16::SP);
  let result = if (offset as u8) & 0x80 == 0 {
    let to_add = offset as u16;
    value.wrapping_add(to_add)
  } else {
    let to_sub = (!(offset as u8) as u16).wrapping_add(1);
    value.wrapping_sub(to_sub)
  };
  let carry = {
    (value & 0xff) + (((offset as u8) as u16) & 0xff) & 0x100 != 0
  };
  let hc = {
    ((value as u8) & 0x0f) + ((offset as u8) & 0x0f) & 0x10 != 0
  };

  set_register_16(registers, Register16::HL, result);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_half_carry(registers, hc);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_rotate_left_a(registers: &mut Registers, length: u32) -> u8 {
  let (_, carry) = rotate_left_through_carry(Register8::A, registers);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_rotate_left_carry_a(registers: &mut Registers, length: u32) -> u8 {
  let (_, carry) = rotate_left(Register8::A, registers);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_rotate_left(reg: Register8, registers: &mut Registers, length: u32) -> u8 {
  let (result, carry) = rotate_left_through_carry(reg, registers);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_rotate_left_indirect(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let address = get_register_16(registers, Register16::HL);
  let prev_carry = ((registers.af & 0x10) >> 4) as u8;
  let value = memory_read_byte(mem, address);
  let new_carry = value & 0x80 == 0x80;
  let result = (value << 1) | prev_carry;
  memory_write_byte(mem, address, result);
  apply_mask(registers, 0xf0);
  test_carry(registers, new_carry);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_rotate_left_carry(reg: Register8, registers: &mut Registers, length: u32) -> u8 {
  let (result, carry) = rotate_left(reg, registers);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_rotate_left_carry_indirect(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let address = get_register_16(registers, Register16::HL);
  let value = memory_read_byte(mem, address);
  let carry_bit = (value & 0x80) >> 7;
  let has_carry = carry_bit != 0;
  let result = (value << 1) | carry_bit;
  memory_write_byte(mem, address, result);
  apply_mask(registers, 0xf0);
  test_carry(registers, has_carry);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_rotate_right_a(registers: &mut Registers, length: u32) -> u8 {
  let (_, carry) = rotate_right_through_carry(Register8::A, registers);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_rotate_right_carry_a(registers: &mut Registers, length: u32) -> u8 {
  let (_, carry) = rotate_right(Register8::A, registers);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_rotate_right(reg: Register8, registers: &mut Registers, length: u32) -> u8 {
  let (result, carry) = rotate_right_through_carry(reg, registers);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_rotate_right_indirect(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let address = get_register_16(registers, Register16::HL);
  let prev_carry = ((registers.af & 0x10) << 3) as u8;
  let value = memory_read_byte(mem, address);
  let new_carry = value & 0x01 == 0x01;
  let result = (value >> 1) | prev_carry;
  memory_write_byte(mem, address, result);
  apply_mask(registers, 0xf0);
  test_carry(registers, new_carry);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_rotate_right_carry(reg: Register8, registers: &mut Registers, length: u32) -> u8 {
  let (result, carry) = rotate_right(reg, registers);
  apply_mask(registers, 0xf0);
  test_carry(registers, carry);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_rotate_right_carry_indirect(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let address = get_register_16(registers, Register16::HL);
  let value = memory_read_byte(mem, address);
  let carry_bit = (value & 0x01) << 7;
  let has_carry = carry_bit != 0;
  let result = (value >> 1) | carry_bit;
  memory_write_byte(mem, address, result);
  apply_mask(registers, 0xf0);
  test_carry(registers, has_carry);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_shift_left(reg: Register8, registers: &mut Registers, length: u32) -> u8 {
  let value = get_register(registers, reg);
  let has_carry = value & 0x80 != 0;
  let result = value << 1;
  set_register(registers, reg, result);
  apply_mask(registers, 0xf0);
  test_carry(registers, has_carry);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_shift_left_indirect(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let address = get_register_16(registers, Register16::HL);
  let value = memory_read_byte(mem, address);
  let has_carry = value & 0x80 != 0;
  let result = value << 1;
  memory_write_byte(mem, address, result);
  apply_mask(registers, 0xf0);
  test_carry(registers, has_carry);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_shift_right(reg: Register8, registers: &mut Registers, length: u32) -> u8 {
  let value = get_register(registers, reg);
  let high_bit = value & 0x80;
  let has_carry = value & 0x01 != 0;
  let result = (value >> 1) | high_bit;
  set_register(registers, reg, result);
  apply_mask(registers, 0xf0);
  test_carry(registers, has_carry);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_shift_right_indirect(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let address = get_register_16(registers, Register16::HL);
  let value = memory_read_byte(mem, address);
  let high_bit = value & 0x80;
  let has_carry = value & 0x01 != 0;
  let result = (value >> 1) | high_bit;
  memory_write_byte(mem, address, result);
  apply_mask(registers, 0xf0);
  test_carry(registers, has_carry);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_shift_right_logical(reg: Register8, registers: &mut Registers, length: u32) -> u8 {
  let value = get_register(registers, reg);
  let has_carry = value & 0x01 != 0;
  let result = value >> 1;
  set_register(registers, reg, result);
  apply_mask(registers, 0xf0);
  test_carry(registers, has_carry);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_shift_right_logical_indirect(registers: &mut Registers, mem: *mut MemoryAreas, length: u32) -> u8 {
  let address = get_register_16(registers, Register16::HL);
  let value = memory_read_byte(mem, address);
  let has_carry = value & 0x01 != 0;
  let result = value >> 1;
  memory_write_byte(mem, address, result);
  apply_mask(registers, 0xf0);
  test_carry(registers, has_carry);
  test_zero(registers, result);
  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_daa(registers: &mut Registers, length: u32) -> u8 {
  let a = get_register(registers, Register8::A);
  if
    (registers.af & 0x20 != 0) ||
    (registers.af & 0x40 == 0 && a & 0x0f > 0x09) {
    // fix low
    if registers.af & 0x40 == 0 {
      registers.af += 0x0600;
    } else {
      registers.af = registers.af.wrapping_sub(0x0600);
      if registers.af & 0x10 == 0 {
        registers.af &= 0xffff;
      }
    }
  }

  if
    (registers.af & 0x10 != 0) ||
    (registers.af & 0x40 == 0 && registers.af & 0xffff00 > 0x9f00) {
    // fix high
    if registers.af & 0x40 == 0 {
      registers.af += 0x6000;
    } else {
      registers.af = registers.af.wrapping_sub(0x6000);
    }
  }


  if registers.af > 0xffff {
    registers.af |= 0x10;
  }
  registers.af &= 0xff50;
  if registers.af & 0xff00 == 0 {
    registers.af |= 0x80;
  }

  registers.ip += length;
  cpu::STATUS_NORMAL
}

fn interp_jump(cond: JumpCondition, addr: u16, registers: &mut Registers) -> u8 {
  match cond {
    JumpCondition::Always => {
      registers.ip = addr as u32;
    },
    JumpCondition::Zero => {
      if registers.af & 0x80 != 0 {
        registers.ip = addr as u32;
        registers.cycles += 1;
      } else {
        registers.ip += 3;
      }
    },
    JumpCondition::Carry => {
      if registers.af & 0x10 != 0 {
        registers.ip = addr as u32;
        registers.cycles += 1;
      } else {
        registers.ip += 3;
      }
    },
    JumpCondition::NonZero => {
      if registers.af & 0x80 == 0 {
        registers.ip = addr as u32;
        registers.cycles += 1;
      } else {
        registers.ip += 3;
      }
    },
    JumpCondition::NoCarry => {
      if registers.af & 0x10 == 0 {
        registers.ip = addr as u32;
        registers.cycles += 1;
      } else {
        registers.ip += 3;
      }
    },
  }
  cpu::STATUS_NORMAL
}

fn interp_jump_hl(registers: &mut Registers) -> u8 {
  let addr = get_register_16(registers, Register16::HL);
  registers.ip = addr as u32;
  cpu::STATUS_NORMAL
}

fn interp_jump_relative(cond: JumpCondition, offset: i8, registers: &mut Registers) -> u8 {
  let should_jump = match cond {
    JumpCondition::Always => true,
    JumpCondition::Zero => registers.af & 0x80 != 0,
    JumpCondition::Carry => registers.af & 0x10 != 0,
    JumpCondition::NonZero => registers.af & 0x80 == 0,
    JumpCondition::NoCarry => registers.af & 0x10 == 0,
  };

  registers.ip = registers.ip.wrapping_add(2);

  if should_jump {
    if (offset as u8) & 0x80 == 0 {
      registers.ip = registers.ip.wrapping_add(offset as u32);
    } else {
      let delta = (!(offset as u8) as u16).wrapping_add(1);
      registers.ip = registers.ip.wrapping_sub(delta as u32);
    }
    registers.cycles += 1;
  }
  cpu::STATUS_NORMAL
}

fn interp_call(cond: JumpCondition, addr: u16, registers: &mut Registers, mem: *mut MemoryAreas) -> u8 {
  let should_call = match cond {
    JumpCondition::Always => true,
    JumpCondition::Zero => registers.af & 0x80 != 0,
    JumpCondition::Carry => registers.af & 0x10 != 0,
    JumpCondition::NonZero => registers.af & 0x80 == 0,
    JumpCondition::NoCarry => registers.af & 0x10 == 0,
  };
  registers.ip = registers.ip.wrapping_add(3);

  if should_call {
    push(registers.ip as u16, registers, mem);
    registers.ip = addr as u32;
    registers.cycles += 3;
  }
  cpu::STATUS_NORMAL
}

fn interp_reset(vector: u16, registers: &mut Registers, mem: *mut MemoryAreas) -> u8 {
  registers.ip = registers.ip.wrapping_add(1);
  push(registers.ip as u16, registers, mem);
  registers.ip = vector as u32;
  cpu::STATUS_NORMAL
}

fn interp_return(cond: JumpCondition, registers: &mut Registers, mem: *mut MemoryAreas) -> u8 {
  let should_return = match cond {
    JumpCondition::Always => true,
    JumpCondition::Zero => registers.af & 0x80 != 0,
    JumpCondition::Carry => registers.af & 0x10 != 0,
    JumpCondition::NonZero => registers.af & 0x80 == 0,
    JumpCondition::NoCarry => registers.af & 0x10 == 0,
  };
  registers.ip += 1;
  if should_return {
    let addr = pop(registers, mem);
    registers.ip = addr as u32;
    registers.cycles += 3;
  }
  cpu::STATUS_NORMAL
}

fn interp_reti(registers: &mut Registers, mem: *mut MemoryAreas) -> u8 {
  let addr = pop(registers, mem);
  registers.ip = addr as u32;
  cpu::STATUS_INTERRUPT_ENABLE
}

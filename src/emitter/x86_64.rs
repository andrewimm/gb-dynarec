use crate::cpu;
use crate::decoder::ops::{Op, IndirectLocation, JumpCondition, Register8, Register16};
use crate::mem::MemoryAreas;

// Register Usage
// When running compiled code, the emulator keeps all GB CPU state in registers.
// Compiled code blocks use the System V 64-bit ABI (even on Windows)
//
// X86  |  GB / Emulator State
// ---------------------------
// RAX  |  AF
// RBX  |  BC
// RDX  |  DE
// RCX  |  HL
// R8   |  SP
// R9   |  IP
// R10  |  Code block return state
// R11  |  Accumulated CPU cycles

pub struct Emitter {
  mem: *const MemoryAreas,
}

impl Emitter {
  pub fn new(mem: *const MemoryAreas) -> Self {
    Self {
      mem
    }
  }

  pub fn encode_prelude(&self, exec: &mut [u8]) -> usize {
    let code = [
      // preserve scratch registers that will be modified
      0x53, // push rbx
      // set initial return code
      0x4d, 0x31, 0xd2, // xor r10, r10
      // begin method, load all registers from a struct in memory
      // the only argument (rdi) will be a pointer to the struct
      0x8b, 0x07, // mov eax, [rdi]
      0x8b, 0x5f, 0x04, // mov ebx, [rdi + 4]
      0x8b, 0x57, 0x08, // mov edx, [rdi + 8]
      0x8b, 0x4f, 0x0c, // mov ecx, [rdi + 12]
      0x66, 0x44, 0x8b, 0x47, 0x10, // mov r8w, [rdi + 16]
      0x66, 0x44, 0x8b, 0x4f, 0x14, // mov r9w, [rdi + 20]
      0x66, 0x44, 0x8b, 0x5f, 0x18, // mov r11w, [rdi + 24]
      0x57, // push rdi
    ];
    let length = code.len();
    exec[..length].copy_from_slice(&code);
    length
  }

  pub fn encode_epilogue(&self, exec: &mut [u8]) -> usize {
    let code = [
      // restore the registers to the struct before returning
      0x5f, // pop rdi
      0x89, 0x07, // mov [rdi], eax
      0x89, 0x5f, 0x04, // mov [rdi + 4], ebx
      0x89, 0x57, 0x08, // mov [rdi + 8], edx
      0x89, 0x4f, 0x0c, // mov [rdi + 12], ecx
      0x66, 0x44, 0x89, 0x47, 0x10, // mov [rdi + 16], r8w
      0x66, 0x44, 0x89, 0x4f, 0x14, // mov [rdi + 20], r9w
      0x66, 0x44, 0x89, 0x5f, 0x18, // mov [rdi + 24], r11w
      // Set return value from r10
      0x4c, 0x89, 0xd0, // mov rax, r10
      // Restore scratch registers to their original value
      0x5b, // pop rbx
      0xc3, // retq
    ];
    let length = code.len();
    exec[..length].copy_from_slice(&code);
    length
  }

  pub fn encode_op(&self, op: Op, ip_increment: usize, exec: &mut [u8]) -> usize {
    match op {
      Op::NoOp => self.encode_noop(exec),
      Op::Load8(dest, src) => self.encode_load_8_register(dest, src, ip_increment, exec),
      Op::Load16(reg, value) => self.encode_load_16(reg, value, ip_increment, exec),
      Op::LoadToIndirect(location, value) => self.encode_load_to_indirect(location, value, ip_increment, exec),
      Op::LoadImmediateToHLIndirect(value) => self.endcode_load_immediate_to_hl_indirect(value, ip_increment, exec),
      Op::LoadFromIndirect(reg, location) => self.encode_load_from_indirect(reg, location, ip_increment, exec),
      Op::Load8Immediate(reg, value) => self.encode_load_8(reg, value, ip_increment, exec),
      Op::Increment8(reg) => self.encode_increment_8(reg, ip_increment, exec),
      Op::Decrement8(reg) => self.encode_decrement_8(reg, ip_increment, exec),
      Op::Increment16(reg) => self.encode_increment_16(reg, ip_increment, exec),
      Op::Decrement16(reg) => self.encode_decrement_16(reg, ip_increment, exec),
      Op::IncrementHLIndirect => self.encode_increment_hl_indirect(ip_increment, exec),
      Op::DecrementHLIndirect => self.encode_decrement_hl_indirect(ip_increment, exec),
      Op::Add8(dest, src) => self.encode_add_register_8(dest, src, ip_increment, exec),
      Op::AddWithCarry8(dest, src) => self.encode_add_register_8_with_carry(dest, src, ip_increment, exec),
      Op::AddHL(src) => self.encode_add_hl(src, ip_increment, exec),
      Op::AddAbsolute8(value) => self.encode_add_absolute_8(value, ip_increment, exec),
      Op::AddAbsoluteWithCarry8(value) => self.encode_adc_absolute_8(value, ip_increment, exec),
      Op::AddIndirect => self.encode_add_indirect(ip_increment, exec),
      Op::AddIndirectWithCarry => self.encode_add_indirect_with_carry(ip_increment, exec),
      Op::Sub8(dest, src) => self.encode_sub_register_8(dest, src, ip_increment, exec),
      Op::SubWithCarry8(dest, src) => self.encode_sub_register_8_with_carry(dest, src, ip_increment, exec),
      Op::SubAbsolute8(value) => self.encode_sub_absolute_8(value, ip_increment, exec),
      Op::SubAbsoluteWithCarry8(value) => self.encode_sbc_absolute_8(value, ip_increment, exec),
      Op::SubIndirect => self.encode_sub_indirect(ip_increment, exec),
      Op::SubIndirectWithCarry => self.encode_sub_indirect_with_carry(ip_increment, exec),
      Op::And8(dest, src) => self.encode_and_register_8(dest, src, ip_increment, exec),
      Op::AndAbsolute8(value) => self.encode_and_absolute_8(value, ip_increment, exec),
      Op::AndIndirect => self.encode_and_indirect(ip_increment, exec),
      Op::Xor8(dest, src) => self.encode_xor_register_8(dest, src, ip_increment, exec),
      Op::XorAbsolute8(value) => self.encode_xor_absolute_8(value, ip_increment, exec),
      Op::XorIndirect => self.encode_xor_indirect(ip_increment, exec),
      Op::Or8(dest, src) => self.encode_or_register_8(dest, src, ip_increment, exec),
      Op::OrAbsolute8(value) => self.encode_or_absolute_8(value, ip_increment, exec),
      Op::OrIndirect => self.encode_or_indirect(ip_increment, exec),
      Op::Compare8(reg) => self.encode_compare(reg, ip_increment, exec),
      Op::CompareIndirect => self.encode_compare_indirect(ip_increment, exec),
      Op::CompareAbsolute8(value) => self.encode_cmp_absolute_8(value, ip_increment, exec),
      Op::RotateLeftA => self.encode_rotate_left_a(ip_increment, exec),
      Op::RotateLeftCarryA => self.encode_rotate_left_carry_a(ip_increment, exec),
      Op::RotateLeft(reg) => self.encode_rotate_left(reg, ip_increment, exec),
      Op::RotateLeftCarry(reg) => self.encode_rotate_left_carry(reg, ip_increment, exec),
      Op::RotateRightA => self.encode_rotate_right_a(ip_increment, exec),
      Op::RotateRightCarryA => self.encode_rotate_right_carry_a(ip_increment, exec),
      Op::RotateRight(reg) => self.encode_rotate_right(reg, ip_increment, exec),
      Op::RotateRightCarry(reg) => self.encode_rotate_right_carry(reg, ip_increment, exec),
      Op::ShiftLeft(reg) => self.encode_shift_left(reg, ip_increment, exec),
      Op::ShiftLeftIndirect => self.encode_shift_left_indirect(ip_increment, exec),
      Op::ShiftRight(reg) => self.encode_shift_right(reg, ip_increment, exec),
      Op::ShiftRightIndirect => self.encode_shift_right_indirect(ip_increment, exec),
      Op::ShiftRightLogical(reg) => self.encode_shift_right_logical(reg, ip_increment, exec),
      Op::ShiftRightLogicalIndirect => self.encode_shift_right_logical_indirect(ip_increment, exec),
      Op::ComplementA => self.encode_complement_a(ip_increment, exec),
      Op::SetCarryFlag => self.encode_set_carry(ip_increment, exec),
      Op::ComplementCarryFlag => self.encode_complement_carry(ip_increment, exec),
      Op::BitSet(reg, mask) => self.encode_bit_set(reg, mask, ip_increment, exec),
      Op::BitClear(reg, mask) => self.encode_bit_clear(reg, mask, ip_increment, exec),
      Op::BitTest(reg, mask) => self.encode_bit_test(reg, mask, ip_increment, exec),
      Op::Swap(reg) => self.encode_swap(reg, ip_increment, exec),
      Op::LoadStackPointerToMemory(addr) => self.encode_load_stack_to_memory(addr, ip_increment, exec),
      Op::LoadAToMemory(addr) => self.encode_load_a_to_memory(addr, ip_increment, exec),
      Op::LoadAFromMemory(addr) => self.encode_load_a_from_memory(addr, ip_increment, exec),
      Op::LoadToHighMem => self.encode_load_to_high_mem(ip_increment, exec),
      Op::LoadFromHighMem => self.encode_load_from_high_mem(ip_increment, exec),
      Op::Push(reg) => self.encode_push(reg, ip_increment, exec),
      Op::Pop(reg) => self.encode_pop(reg, ip_increment, exec),
      Op::AddSP(offset) => self.encode_add_sp(offset, ip_increment, exec),
      Op::LoadToStackPointer => self.encode_load_to_sp(ip_increment, exec),
      Op::LoadStackOffset(offset) => self.encode_load_stack_offset(offset, ip_increment, exec),
      Op::DAA => self.encode_daa(ip_increment, exec),

      Op::Jump(cond, address) => self.encode_jump(cond, address, exec),
      Op::JumpHL => self.encode_jump_hl(exec),
      Op::JumpRelative(cond, offset) => self.encode_jump_relative(cond, offset, exec),
      Op::Call(cond, address) => self.encode_call(cond, address, exec),
      Op::ResetVector(vector) => self.encode_reset(vector, exec),
      Op::Return(cond) => self.encode_return(cond, exec),
      Op::ReturnFromInterrupt => self.encode_return_from_interrupt(exec),

      Op::Stop => self.encode_stop(ip_increment, exec),
      Op::Halt => self.encode_halt(ip_increment, exec),
      Op::InterruptEnable => self.encode_interrupt_enable(ip_increment, exec),
      Op::InterruptDisable => self.encode_interrupt_disable(ip_increment, exec),

      Op::Invalid(code) => panic!("Invalid OP: {:#04x}", code),
    }
  }

  pub fn encode_noop(&self, exec: &mut [u8]) -> usize {
    emit_ip_increment(1, exec)
  }

  pub fn encode_stop(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_return_code(cpu::STATUS_STOP, exec);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_halt(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_return_code(cpu::STATUS_HALT, exec);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_interrupt_enable(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_return_code(cpu::STATUS_INTERRUPT_ENABLE, exec);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_interrupt_disable(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_return_code(cpu::STATUS_INTERRUPT_DISABLE, exec);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_load_16(&self, dest: Register16, value: u16, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_move_16(map_register_16(dest), value, exec);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_load_8(&self, dest: Register8, value: u8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_move_8(map_register_8(dest), value, exec);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_load_8_register(&self, dest: Register8, src: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_reg_to_reg_move(map_register_8(dest), map_register_8(src), exec);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_increment_8(&self, dest: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_increment_8(map_register_8(dest), exec);
    len += emit_store_flags(0xe0, false, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_decrement_8(&self, dest: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_decrement_8(map_register_8(dest), exec);
    len += emit_store_flags(0xe0, true, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_increment_16(&self, dest: Register16, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_increment_16(map_register_16(dest), exec);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_decrement_16(&self, dest: Register16, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_decrement_16(map_register_16(dest), exec);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_increment_hl_indirect(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_hl_indirect_partial_read(self.mem as usize, exec);
    len += emit_increment_8(X86Reg8::DL, &mut exec[len..]);
    len += emit_store_flags(0xe0, false, &mut exec[len..]);
    len += emit_hl_indirect_partial_write(self.mem as usize, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_decrement_hl_indirect(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_hl_indirect_partial_read(self.mem as usize, exec);
    len += emit_decrement_8(X86Reg8::DL, &mut exec[len..]);
    len += emit_store_flags(0xe0, true, &mut exec[len..]);
    len += emit_hl_indirect_partial_write(self.mem as usize, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_add_register_8(&self, dest: Register8, src: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_add_register_8(map_register_8(dest), map_register_8(src), exec);
    len += emit_store_flags(0xf0, false, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_add_register_8_with_carry(&self, dest: Register8, src: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_restore_carry(exec);
    len += emit_add_register_8_with_carry(map_register_8(dest), map_register_8(src), &mut exec[len..]);
    len += emit_store_flags(0xf0, false, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_add_indirect(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_push_register(X86Reg64::RDX, exec);
    len += emit_hl_indirect_read(self.mem as usize, &mut exec[len..]);
    len += emit_add_register_8(X86Reg8::AH, X86Reg8::DL, &mut exec[len..]);
    len += emit_store_flags(0xf0, false, &mut exec[len..]);
    len += emit_pop_register(X86Reg64::RDX, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_add_indirect_with_carry(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_push_register(X86Reg64::RDX, exec);
    len += emit_hl_indirect_read(self.mem as usize, &mut exec[len..]);
    len += emit_restore_carry(&mut exec[len..]);
    len += emit_add_register_8_with_carry(X86Reg8::AH, X86Reg8::DL, &mut exec[len..]);
    len += emit_store_flags(0xf0, false, &mut exec[len..]);
    len += emit_pop_register(X86Reg64::RDX, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_sub_register_8(&self, dest: Register8, src: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_sub_register_8(map_register_8(dest), map_register_8(src), exec);
    len += emit_store_flags(0xf0, true, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_sub_register_8_with_carry(&self, dest: Register8, src: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_restore_carry(exec);
    len += emit_sub_register_8_with_carry(map_register_8(dest), map_register_8(src), &mut exec[len..]);
    len += emit_store_flags(0xf0, true, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_sub_indirect(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_push_register(X86Reg64::RDX, exec);
    len += emit_hl_indirect_read(self.mem as usize, &mut exec[len..]);
    len += emit_sub_register_8(X86Reg8::AH, X86Reg8::DL, &mut exec[len..]);
    len += emit_store_flags(0xf0, true, &mut exec[len..]);
    len += emit_pop_register(X86Reg64::RDX, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_sub_indirect_with_carry(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_push_register(X86Reg64::RDX, exec);
    len += emit_hl_indirect_read(self.mem as usize, &mut exec[len..]);
    len += emit_restore_carry(&mut exec[len..]);
    len += emit_sub_register_8_with_carry(X86Reg8::AH, X86Reg8::DL, &mut exec[len..]);
    len += emit_store_flags(0xf0, true, &mut exec[len..]);
    len += emit_pop_register(X86Reg64::RDX, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_and_register_8(&self, dest: Register8, src: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_and_register_8(map_register_8(dest), map_register_8(src), exec);
    len += emit_store_flags(0x80, false, &mut exec[len..]);
    len += emit_force_flags_off(0x05, &mut exec[len..]);
    len += emit_force_flags_on(0x20, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_and_indirect(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_push_register(X86Reg64::RDX, exec);
    len += emit_hl_indirect_read(self.mem as usize, &mut exec[len..]);
    len += emit_and_register_8(X86Reg8::AH, X86Reg8::DL, &mut exec[len..]);
    len += emit_store_flags(0x80, false, &mut exec[len..]);
    len += emit_force_flags_off(0x05, &mut exec[len..]);
    len += emit_force_flags_on(0x20, &mut exec[len..]);
    len += emit_pop_register(X86Reg64::RDX, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_or_register_8(&self, dest: Register8, src: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_or_register_8(map_register_8(dest), map_register_8(src), exec);
    len += emit_store_flags(0x80, false, &mut exec[len..]);
    len += emit_force_flags_off(0x07, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_or_indirect(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_push_register(X86Reg64::RDX, exec);
    len += emit_hl_indirect_read(self.mem as usize, &mut exec[len..]);
    len += emit_or_register_8(X86Reg8::AH, X86Reg8::DL, &mut exec[len..]);
    len += emit_store_flags(0x80, false, &mut exec[len..]);
    len += emit_force_flags_off(0x07, &mut exec[len..]);
    len += emit_pop_register(X86Reg64::RDX, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_xor_register_8(&self, dest: Register8, src: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_xor_register_8(map_register_8(dest), map_register_8(src), exec);
    len += emit_store_flags(0x80, false, &mut exec[len..]);
    len += emit_force_flags_off(0x07, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_xor_indirect(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_push_register(X86Reg64::RDX, exec);
    len += emit_hl_indirect_read(self.mem as usize, &mut exec[len..]);
    len += emit_xor_register_8(X86Reg8::AH, X86Reg8::DL, &mut exec[len..]);
    len += emit_store_flags(0x80, false, &mut exec[len..]);
    len += emit_force_flags_off(0x07, &mut exec[len..]);
    len += emit_pop_register(X86Reg64::RDX, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_compare(&self, reg: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_compare(map_register_8(reg), exec);
    len += emit_store_flags(0xf0, true, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_compare_indirect(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_push_register(X86Reg64::RDX, exec);
    len += emit_hl_indirect_read(self.mem as usize, &mut exec[len..]);
    len += emit_compare(X86Reg8::DL, &mut exec[len..]);
    len += emit_store_flags(0xf0, true, &mut exec[len..]);
    len += emit_pop_register(X86Reg64::RDX, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_add_absolute_8(&self, value: u8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_add_absolute_8(value, exec);
    len += emit_store_flags(0xf0, false, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_adc_absolute_8(&self, value: u8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_restore_carry(exec);
    len += emit_adc_absolute_8(value, &mut exec[len..]);
    len += emit_store_flags(0xf0, false, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_sub_absolute_8(&self, value: u8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_sub_absolute_8(value, exec);
    len += emit_store_flags(0xf0, true, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_sbc_absolute_8(&self, value: u8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_sbc_absolute_8(value, exec);
    len += emit_store_flags(0xf0, true, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_and_absolute_8(&self, value: u8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_and_absolute_8(value, exec);
    len += emit_store_flags(0xc0, false, &mut exec[len..]);
    len += emit_force_flags_off(0x10, &mut exec[len..]);
    len += emit_force_flags_on(0x20, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_or_absolute_8(&self, value: u8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_or_absolute_8(value, exec);
    len += emit_store_flags(0x80, false, &mut exec[len..]);
    len += emit_force_flags_off(0x70, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_xor_absolute_8(&self, value: u8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_xor_absolute_8(value, exec);
    len += emit_store_flags(0x80, false, &mut exec[len..]);
    len += emit_force_flags_off(0x70, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_cmp_absolute_8(&self, value: u8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_cmp_absolute_8(value, exec);
    len += emit_store_flags(0xf0, true, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_add_hl(&self, src: Register16, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_add_hl(map_register_16(src), exec);
    len += emit_store_flags(0x70, false, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_rotate_left_a(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_restore_carry(exec);
    len += emit_rotate_left_through_carry(X86Reg8::AH, &mut exec[len..]);
    len += emit_store_flags(0x10, false, &mut exec[len..]);
    len += emit_force_flags_off(0xe0, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_rotate_left(&self, reg: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_restore_carry(exec);
    len += emit_rotate_left_through_carry(map_register_8(reg), &mut exec[len..]);
    len += emit_store_flags(0x90, false, &mut exec[len..]);
    len += emit_force_flags_off(0x60, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_rotate_left_carry_a(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_rotate_left(X86Reg8::AH, exec);
    len += emit_store_flags(0x10, false, &mut exec[len..]);
    len += emit_force_flags_off(0xe0, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_rotate_left_carry(&self, reg: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let xreg = map_register_8(reg);
    let mut len = emit_rotate_left(xreg, exec);
    len += emit_store_flags(0x10, false, &mut exec[len..]);
    len += emit_zero_flag_test(xreg, &mut exec[len..]);
    len += emit_force_flags_off(0x60, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_rotate_right_a(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_restore_carry(exec);
    len += emit_rotate_right_through_carry(X86Reg8::AH, &mut exec[len..]);
    len += emit_store_flags(0x10, false, &mut exec[len..]);
    len += emit_force_flags_off(0xe0, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_rotate_right(&self, reg: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_restore_carry(exec);
    len += emit_rotate_right_through_carry(map_register_8(reg), &mut exec[len..]);
    len += emit_store_flags(0x90, false, &mut exec[len..]);
    len += emit_force_flags_off(0x60, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_rotate_right_carry_a(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_rotate_right(X86Reg8::AH, exec);
    len += emit_store_flags(0x10, false, &mut exec[len..]);
    len += emit_force_flags_off(0xe0, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_rotate_right_carry(&self, reg: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_rotate_right(map_register_8(reg), exec);
    len += emit_store_flags(0x10, false, &mut exec[len..]);
    len += emit_force_flags_off(0xe0, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_shift_left(&self, reg: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_shift_left(map_register_8(reg), exec);
    len += emit_store_flags(0x90, false, &mut exec[len..]);
    len += emit_force_flags_off(0x60, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_shift_left_indirect(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_hl_indirect_partial_read(self.mem as usize, exec);
    len += emit_shift_left(X86Reg8::DL, &mut exec[len..]);
    len += emit_store_flags(0x90, false, &mut exec[len..]);
    len += emit_force_flags_off(0x60, &mut exec[len..]);
    len += emit_hl_indirect_partial_write(self.mem as usize, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_shift_right(&self, reg: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_shift_right(map_register_8(reg), exec);
    len += emit_store_flags(0x90, false, &mut exec[len..]);
    len += emit_force_flags_off(0x60, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_shift_right_indirect(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_hl_indirect_partial_read(self.mem as usize, exec);
    len += emit_shift_right(X86Reg8::DL, &mut exec[len..]);
    len += emit_store_flags(0x90, false, &mut exec[len..]);
    len += emit_force_flags_off(0x60, &mut exec[len..]);
    len += emit_hl_indirect_partial_write(self.mem as usize, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_shift_right_logical(&self, reg: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_shift_right_logical(map_register_8(reg), exec);
    len += emit_store_flags(0x90, false, &mut exec[len..]);
    len += emit_force_flags_off(0x60, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_shift_right_logical_indirect(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_hl_indirect_partial_read(self.mem as usize, exec);
    len += emit_shift_right_logical(X86Reg8::DL, &mut exec[len..]);
    len += emit_store_flags(0x90, false, &mut exec[len..]);
    len += emit_force_flags_off(0x60, &mut exec[len..]);
    len += emit_hl_indirect_partial_write(self.mem as usize, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_complement_a(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_complement_a(exec);
    len += emit_force_flags_on(0x60, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_set_carry(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_force_flags_off(0x60, exec);
    len += emit_force_flags_on(0x10, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_complement_carry(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_force_flags_off(0x60, exec);
    len += emit_complement_carry(&mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_bit_set(&self, reg: Register8, mask: u8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_register_or(map_register_8(reg), mask, exec);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_bit_clear(&self, reg: Register8, mask: u8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_register_and(map_register_8(reg), !mask, exec);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_bit_test(&self, reg: Register8, mask: u8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_bit_test(map_register_8(reg), mask, exec); 
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_swap(&self, reg: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let x86_reg = map_register_8(reg);
    let mut len = emit_swap(x86_reg, exec);
    len += emit_or_register_8(x86_reg, x86_reg, &mut exec[len..]);
    len += emit_store_flags(0x80, false, &mut exec[len..]);
    len += emit_force_flags_off(0x70, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_load_to_indirect(&self, location: IndirectLocation, value: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let indirect_address = map_indirect_location_to_register(location);
    let mut len= emit_memory_write(exec, self.mem as usize, indirect_address, map_register_8(value));
    len += match location {
      IndirectLocation::HLIncrement => emit_increment_16(X86Reg16::CX, &mut exec[len..]),
      IndirectLocation::HLDecrement => emit_decrement_16(X86Reg16::CX, &mut exec[len..]),
      _ => 0,
    };
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn endcode_load_immediate_to_hl_indirect(&self, value: u8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let indirect_address = map_indirect_location_to_register(IndirectLocation::HL);
    let len = emit_memory_write_literal(exec, self.mem as usize, indirect_address, value);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_load_from_indirect(&self, reg: Register8, location: IndirectLocation, ip_increment: usize, exec: &mut [u8]) -> usize {
    let indirect_address = map_indirect_location_to_register(location);
    let dest_register = map_register_8(reg);
    let mut len = emit_memory_read(exec, self.mem as usize, indirect_address, dest_register);
    len += match location {
      IndirectLocation::HLIncrement => emit_increment_16(X86Reg16::CX, &mut exec[len..]),
      IndirectLocation::HLDecrement => emit_decrement_16(X86Reg16::CX, &mut exec[len..]),
      _ => 0,
    };
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_load_stack_to_memory(&self, addr: u16, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_write_stack_to_memory(exec, self.mem as usize, addr);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_load_a_to_memory(&self, addr: u16, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_write_a_to_memory(exec, self.mem as usize, addr);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_load_a_from_memory(&self, addr: u16, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_read_a_from_memory(exec, self.mem as usize, addr);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_load_to_high_mem(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_load_to_high_mem(exec, self.mem as usize);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_load_from_high_mem(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_load_from_high_mem(exec, self.mem as usize);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_push(&self, reg: Register16, ip_increment: usize, exec: &mut [u8]) -> usize {
    let source = map_register_16(reg);
    let len = emit_push(source, self.mem as usize, exec);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_pop(&self, reg: Register16, ip_increment: usize, exec: &mut [u8]) -> usize {
    let dest = map_register_16(reg);
    let len = emit_pop(dest, self.mem as usize, exec);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_add_sp(&self, offset: i8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_sp_signed_offset(offset, exec);
    len += emit_store_flags(0x70, false, &mut exec[len..]);
    len += emit_force_flags_off(0x80, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_load_to_sp(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_load_to_sp(exec);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_load_stack_offset(&self, offset: i8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_load_stack_offset(offset, exec);
    len += emit_store_flags(0x70, false, &mut exec[len..]);
    len += emit_force_flags_off(0x80, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_daa(&self, ip_increment: usize, exec: &mut [u8]) -> usize {
    let len = emit_daa(exec);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_jump(&self, condition: JumpCondition, address: u16, exec: &mut [u8]) -> usize {
    let mut len;
    match condition {
      JumpCondition::Always => return emit_jump(address, exec),
      // On the GB, a conditional jump either changes the IP to an entirely new
      // address, or it increments it to the next instruction.
      // A Jump will end a code block, so this instruction doesn't need to
      // "jump" on the host processor. It only needs to change the IP register
      // and return.
      // To implement this, all code goes through the default fallthrough path
      // incrementing the IP. Then, it tests the conditional flag for the jump.
      // If that flag is *not* set, it jumps to the epilogue. Otherwise, it
      // first hits an instruction that modifies the IP to the new location.
      JumpCondition::Zero => {
        len = emit_ip_increment(3, exec);
        // test against the zero flag (0x80); if it's set, the host's zero flag
        // will be cleared
        len += emit_flag_test(0x80, &mut exec[len..]);
        // If the condition was set, this flag will be cleared and the jump will
        // fail. It will fall through to the successive instruction, which sets
        // the value of the IP directly.
        len += emit_jump_zero(5, &mut exec[len..]);
        len += emit_move_16(X86Reg16::R9, address, &mut exec[len..]);
      },
      JumpCondition::NonZero => {
        len = emit_ip_increment(3, exec);
        len += emit_flag_test(0x80, &mut exec[len..]);
        len += emit_jump_nonzero(5, &mut exec[len..]);
        len += emit_move_16(X86Reg16::R9, address, &mut exec[len..]);
      },
      JumpCondition::Carry => {
        len = emit_ip_increment(3, exec);
        len += emit_flag_test(0x10, &mut exec[len..]);
        len += emit_jump_zero(5, &mut exec[len..]);
        len += emit_move_16(X86Reg16::R9, address, &mut exec[len..]);
      },
      JumpCondition::NoCarry => {
        len = emit_ip_increment(3, exec);
        len += emit_flag_test(0x10, &mut exec[len..]);
        len += emit_jump_nonzero(5, &mut exec[len..]);
        len += emit_move_16(X86Reg16::R9, address, &mut exec[len..]);
      },
    }
    len
  }

  pub fn encode_jump_relative(&self, condition: JumpCondition, offset: i8, exec: &mut [u8]) -> usize {
    let mut len;
    match condition {
      JumpCondition::Always => {
        len = emit_ip_increment(2, exec);
        len += emit_ip_signed_offset(offset, &mut exec[len..]);
      },
      JumpCondition::Zero => {
        len = emit_ip_increment(2, exec);
        len += emit_flag_test(0x80, &mut exec[len..]);
        len += emit_jump_zero(5, &mut exec[len..]);
        len += emit_ip_signed_offset(offset, &mut exec[len..]);
      },
      JumpCondition::NonZero => {
        len = emit_ip_increment(2, exec);
        len += emit_flag_test(0x80, &mut exec[len..]);
        len += emit_jump_nonzero(5, &mut exec[len..]);
        len += emit_ip_signed_offset(offset, &mut exec[len..]);
      },
      JumpCondition::Carry => {
        len = emit_ip_increment(3, exec);
        len += emit_flag_test(0x10, &mut exec[len..]);
        len += emit_jump_zero(5, &mut exec[len..]);
        len += emit_ip_signed_offset(offset, &mut exec[len..]);
      },
      JumpCondition::NoCarry => {
        len = emit_ip_increment(3, exec);
        len += emit_flag_test(0x10, &mut exec[len..]);
        len += emit_jump_nonzero(5, &mut exec[len..]);
        len += emit_ip_signed_offset(offset, &mut exec[len..]);
      },
    }
    len
  }

  pub fn encode_jump_hl(&self, exec: &mut [u8]) -> usize {
    emit_jump_hl(exec)
  }

  pub fn encode_call(&self, condition: JumpCondition, address: u16, exec: &mut [u8]) -> usize {
    let mut len;
    match condition {
      JumpCondition::Always => {
        len = emit_ip_increment(3, exec);
        len += emit_push(X86Reg16::R9, self.mem as usize, &mut exec[len..]);
        len += emit_jump(address, &mut exec[len..]);
      },
      JumpCondition::Zero => {
        len = emit_ip_increment(3, exec);
        len += emit_flag_test(0x80, &mut exec[len..]);
        len += emit_jump_zero(0, &mut exec[len..]);
        let offset_location = len;
        len += emit_push(X86Reg16::R9, self.mem as usize, &mut exec[len..]);
        len += emit_move_16(X86Reg16::R9, address, &mut exec[len..]);
        let delta = len - offset_location;
        exec[offset_location - 1] = delta as u8;
      },
      JumpCondition::NonZero => {
        len = emit_ip_increment(3, exec);
        len += emit_flag_test(0x80, &mut exec[len..]);
        len += emit_jump_nonzero(0, &mut exec[len..]);
        let offset_location = len;
        len += emit_push(X86Reg16::R9, self.mem as usize, &mut exec[len..]);
        len += emit_move_16(X86Reg16::R9, address, &mut exec[len..]);
        let delta = len - offset_location;
        exec[offset_location - 1] = delta as u8;
      },
      JumpCondition::Carry => {
        len = emit_ip_increment(3, exec);
        len += emit_flag_test(0x10, &mut exec[len..]);
        len += emit_jump_zero(0, &mut exec[len..]);
        let offset_location = len;
        len += emit_push(X86Reg16::R9, self.mem as usize, &mut exec[len..]);
        len += emit_move_16(X86Reg16::R9, address, &mut exec[len..]);
        let delta = len - offset_location;
        exec[offset_location - 1] = delta as u8;
      },
      JumpCondition::NoCarry => {
        len = emit_ip_increment(3, exec);
        len += emit_flag_test(0x10, &mut exec[len..]);
        len += emit_jump_nonzero(0, &mut exec[len..]);
        let offset_location = len;
        len += emit_push(X86Reg16::R9, self.mem as usize, &mut exec[len..]);
        len += emit_move_16(X86Reg16::R9, address, &mut exec[len..]);
        let delta = len - offset_location;
        exec[offset_location - 1] = delta as u8;
      },
    }
    len
  }

  pub fn encode_reset(&self, vector: u16, exec: &mut [u8]) -> usize {
    let mut len = emit_ip_increment(1, exec);
    len += emit_push(X86Reg16::R9, self.mem as usize, &mut exec[len..]);
    len + emit_jump(vector, &mut exec[len..])
  }

  pub fn encode_return(&self, condition: JumpCondition, exec: &mut [u8]) -> usize {
    let mut len;
    match condition {
      JumpCondition::Always => {
        len = emit_pop(X86Reg16::R9, self.mem as usize, exec);
      },
      JumpCondition::Zero => {
        len = emit_ip_increment(1, exec);
        len += emit_flag_test(0x80, &mut exec[len..]);
        len += emit_jump_zero(0, &mut exec[len..]);
        let offset_location = len;
        len += emit_pop(X86Reg16::R9, self.mem as usize, &mut exec[len..]);
        let delta = len - offset_location;
        exec[offset_location - 1] = delta as u8;
      },
      JumpCondition::NonZero => {
        len = emit_ip_increment(1, exec);
        len += emit_flag_test(0x80, &mut exec[len..]);
        len += emit_jump_nonzero(0, &mut exec[len..]);
        let offset_location = len;
        len += emit_pop(X86Reg16::R9, self.mem as usize, &mut exec[len..]);
        let delta = len - offset_location;
        exec[offset_location - 1] = delta as u8;
      },
      JumpCondition::Carry => {
        len = emit_ip_increment(1, exec);
        len += emit_flag_test(0x10, &mut exec[len..]);
        len += emit_jump_zero(0, &mut exec[len..]);
        let offset_location = len;
        len += emit_pop(X86Reg16::R9, self.mem as usize, &mut exec[len..]);
        let delta = len - offset_location;
        exec[offset_location - 1] = delta as u8;
      },
      JumpCondition::NoCarry => {
        len = emit_ip_increment(1, exec);
        len += emit_flag_test(0x10, &mut exec[len..]);
        len += emit_jump_nonzero(0, &mut exec[len..]);
        let offset_location = len;
        len += emit_pop(X86Reg16::R9, self.mem as usize, &mut exec[len..]);
        let delta = len - offset_location;
        exec[offset_location - 1] = delta as u8;
      },
    }
    len
  }

  pub fn encode_return_from_interrupt(&self, exec: &mut [u8]) -> usize {
    let len = emit_pop(X86Reg16::R9, self.mem as usize, exec);
    len + emit_return_code(cpu::STATUS_INTERRUPT_ENABLE, &mut exec[len..])
  }
}

fn emit_immediate_u16(value: u16, exec: &mut [u8]) {
  exec[0] = (value & 0xff) as u8;
  exec[1] = (value >> 8) as u8;
}

fn emit_return_code(code: u8, exec: &mut [u8]) -> usize {
  exec[0] = 0x41; // mov r10b, code
  exec[1] = 0xb2;
  exec[2] = code;
  3
}

fn emit_move_16(dest: X86Reg16, value: u16, exec: &mut [u8]) -> usize {
  exec[0] = 0x66;
  let mut pointer = 1;
  match dest {
    X86Reg16::AX => exec[pointer] = 0xb8,
    X86Reg16::CX => exec[pointer] = 0xb9,
    X86Reg16::DX => exec[pointer] = 0xba,
    X86Reg16::BX => exec[pointer] = 0xbb,
    X86Reg16::R8 => {
      exec[pointer] = 0x41;
      pointer += 1;
      exec[pointer] = 0xb8;
    },
    X86Reg16::R9 => {
      exec[pointer] = 0x41;
      pointer += 1;
      exec[pointer] = 0xb9;
    },
  }
  pointer += 1;
  emit_immediate_u16(value, &mut exec[pointer..]);
  pointer += 2;
  
  pointer
}

fn emit_move_8(dest: X86Reg8, value: u8, exec: &mut [u8]) -> usize {
  match dest {
    X86Reg8::AL => exec[0] = 0xb0,
    X86Reg8::CL => exec[0] = 0xb1,
    X86Reg8::DL => exec[0] = 0xb2,
    X86Reg8::BL => exec[0] = 0xb3,
    X86Reg8::AH => exec[0] = 0xb4,
    X86Reg8::CH => exec[0] = 0xb5,
    X86Reg8::DH => exec[0] = 0xb6,
    X86Reg8::BH => exec[0] = 0xb7,
  }
  exec[1] = value;
  2
}

fn emit_reg_to_reg_move(to: X86Reg8, from: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0x88;
  exec[1] = register_to_register(from, to);
  2
}

fn emit_add_register_8(dest: X86Reg8, src: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0x00;
  exec[1] = register_to_register(src, dest);
  2
}

fn emit_add_register_8_with_carry(dest: X86Reg8, src: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0x10;
  exec[1] = register_to_register(src, dest);
  2
}

fn emit_sub_register_8(dest: X86Reg8, src: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0x28;
  exec[1] = register_to_register(src, dest);
  2
}

fn emit_sub_register_8_with_carry(dest: X86Reg8, src: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0x18;
  exec[1] = register_to_register(src, dest);
  2
}

fn emit_and_register_8(dest: X86Reg8, src: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0x20;
  exec[1] = register_to_register(src, dest);
  2
}

fn emit_or_register_8(dest: X86Reg8, src: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0x08;
  exec[1] = register_to_register(src, dest);
  2
}

fn emit_xor_register_8(dest: X86Reg8, src: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0x30;
  exec[1] = register_to_register(src, dest);
  2
}

fn emit_add_hl(src: X86Reg16, exec: &mut [u8]) -> usize {
  match src {
    X86Reg16::BX => {
      exec[0] = 0x00;
      exec[1] = 0xd9;
      exec[2] = 0x10;
      exec[3] = 0xfd;
      4
    },
    X86Reg16::CX => {
      exec[0] = 0x00;
      exec[1] = 0xc9;
      exec[2] = 0x10;
      exec[3] = 0xed;
      4
    },
    X86Reg16::DX => {
      exec[0] = 0x00;
      exec[1] = 0xd1;
      exec[2] = 0x10;
      exec[3] = 0xf5;
      4
    },
    X86Reg16::R8 => {
      // r8 is messier to deal with because we can't touch bits 8-15 directly
      let code = [
        0x44, 0x00, 0xc1, // add cl, r8b
        0x9c, // pushf
        0xc1, 0xc9, 0x08, // ror ecx, 8
        0x41, 0xc1, 0xc8, 0x08, // ror r8d, 8
        0x9d, // popf
        0x44, 0x10, 0xc1, // adc cl, r8b
        0x9c, // pushf
        0xc1, 0xc1, 0x08, // rol ecx, 8
        0x41, 0xc1, 0xc0, 0x08, // rol r8d, 8
        0x9d, // popf
      ];
      let length = code.len();
      exec[..length].copy_from_slice(&code);
      length
    },
    _ => panic!("Invalid source"),
  }
}

fn emit_add_absolute_8(value: u8, exec: &mut [u8]) -> usize {
  exec[0] = 0x80;
  exec[1] = 0xc4;
  exec[2] = value;
  3
}

fn emit_adc_absolute_8(value: u8, exec: &mut [u8]) -> usize {
  exec[0] = 0x80;
  exec[1] = 0xd4;
  exec[2] = value;
  3
}

fn emit_sub_absolute_8(value: u8, exec: &mut [u8]) -> usize {
  exec[0] = 0x80;
  exec[1] = 0xec;
  exec[2] = value;
  3
}

fn emit_sbc_absolute_8(value: u8, exec: &mut [u8]) -> usize {
  exec[0] = 0x80;
  exec[1] = 0xdc;
  exec[2] = value;
  3
}

fn emit_and_absolute_8(value: u8, exec: &mut [u8]) -> usize {
  exec[0] = 0x80;
  exec[1] = 0xe4;
  exec[2] = value;
  3
}

fn emit_xor_absolute_8(value: u8, exec: &mut [u8]) -> usize {
  exec[0] = 0x80;
  exec[1] = 0xf4;
  exec[2] = value;
  3
}

fn emit_or_absolute_8(value: u8, exec: &mut [u8]) -> usize {
  exec[0] = 0x80;
  exec[1] = 0xcc;
  exec[2] = value;
  3
}

fn emit_compare(other: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0x38;
  exec[1] = register_to_register(other, X86Reg8::AH);
  2
}

fn emit_cmp_absolute_8(value: u8, exec: &mut [u8]) -> usize {
  exec[0] = 0x80;
  exec[1] = 0xfc;
  exec[2] = value;
  3
}

fn emit_force_flags_off(flags: u8, exec: &mut [u8]) -> usize {
  exec[0] = 0x24; // and al, !flags
  exec[1] = !flags;
  2
}

fn emit_force_flags_on(flags: u8, exec: &mut [u8]) -> usize {
  exec[0] = 0x0c; // or al, flags
  exec[1] = flags;
  2
}

fn emit_zero_flag_test(reg: X86Reg8, exec: &mut [u8]) -> usize {
  let code = [
    0x08, register_to_register(reg, reg), // or reg, reg
    0x41, 0x0f, 0x94, 0xc6, // setz sil
    0x41, 0xd0, 0xce, // ror sil
    0x44, 0x08, 0xf0, // or al, sil
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  length
}

fn emit_store_flags(mask: u8, negative: bool, exec: &mut [u8]) -> usize {
  let code = [
    0x9c, // pushfq
    0x5e, // pop rsi ; put flags in rsi
    0x83, 0xe6, 0x51, // and esi, 0b01010001 ; clear everything but Z, A, C
    0xd1, 0xe6, // shl esi
    0x83, 0xc6, 0x0e, // add esi, 0b00001110 ; these two lines effectively move C
    0x81, 0xe6, 0xf0, 0x00, 0x00, 0x00, // and esi, 0b11110000 ; from bit 1 to bit 5
    0x25, !mask, 0xff, 0x00, 0x00, // and eax, (0xff00 | !mask) ; clear all masked bits in F (AL)
    0x81, 0xe6, mask, 0x00, 0x00, 0x00, // and esi, mask ; only set masked bits
    0x09, 0xf0, // or eax, esi ; set the new flags
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  if negative && (mask & 0x40 != 0) {
    // set the negative bit
    exec[length] = 0x0c;
    exec[length + 1] = 0x40;
    length + 2
  } else {
    length
  }
}

/// This destroys $al, it should only be used in methods that modify all flags
fn emit_restore_carry(exec: &mut [u8]) -> usize {
  let code = [
    0x24, 0x10, // and al, 0x10 ; isolate the carry bit
    0x04, 0xf0, // add al, 0xf0 ; if bit 4 is set, this will cause a carry
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  length
}

fn emit_increment_8(dest: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0xfe; // inc dest
  exec[1] = match dest {
    X86Reg8::AL => 0xc0,
    X86Reg8::CL => 0xc1,
    X86Reg8::DL => 0xc2,
    X86Reg8::BL => 0xc3,
    X86Reg8::AH => 0xc4,
    X86Reg8::CH => 0xc5,
    X86Reg8::DH => 0xc6,
    X86Reg8::BH => 0xc7,
  };
  2
}

fn emit_decrement_8(dest: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0xfe;
  exec[1] = match dest {
    X86Reg8::AL => 0xc8,
    X86Reg8::CL => 0xc9,
    X86Reg8::DL => 0xca,
    X86Reg8::BL => 0xcb,
    X86Reg8::AH => 0xcc,
    X86Reg8::CH => 0xcd,
    X86Reg8::DH => 0xce,
    X86Reg8::BH => 0xcf,
  };
  2
}

fn emit_increment_16(dest: X86Reg16, exec: &mut [u8]) -> usize {
  exec[0] = 0x66;
  exec[1] = 0xff;
  exec[2] = match dest {
    X86Reg16::CX => 0xc1,
    X86Reg16::DX => 0xc2,
    X86Reg16::BX => 0xc3,
    _ => panic!("Cannot increment register"),
  };
  3
}

fn emit_decrement_16(dest: X86Reg16, exec: &mut [u8]) -> usize {
  exec[0] = 0x66;
  exec[1] = 0xff;
  exec[2] = match dest {
    X86Reg16::CX => 0xc9,
    X86Reg16::DX => 0xca,
    X86Reg16::BX => 0xcb,
    _ => panic!("Cannot increment register"),
  };
  3
}

fn emit_rotate_left_through_carry(register: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0xd0;
  exec[1] = match register {
    X86Reg8::AH => 0xd4,
    X86Reg8::BH => 0xd7,
    X86Reg8::BL => 0xd3,
    X86Reg8::CH => 0xd5,
    X86Reg8::CL => 0xd1,
    X86Reg8::DH => 0xd6,
    X86Reg8::DL => 0xd2,
    _ => panic!("Cannot rotate register"),
  };
  2
}

fn emit_rotate_left(register: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0xd0;
  exec[1] = match register {
    X86Reg8::AH => 0xc4,
    X86Reg8::BH => 0xc7,
    X86Reg8::BL => 0xc3,
    X86Reg8::CH => 0xc5,
    X86Reg8::CL => 0xc1,
    X86Reg8::DH => 0xc6,
    X86Reg8::DL => 0xc2,
    _ => panic!("Cannot rotate register"),
  };
  2
}

fn emit_rotate_right_through_carry(register: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0xd0;
  exec[1] = match register {
    X86Reg8::AH => 0xdc,
    X86Reg8::BH => 0xdf,
    X86Reg8::BL => 0xdb,
    X86Reg8::CH => 0xdd,
    X86Reg8::CL => 0xd9,
    X86Reg8::DH => 0xde,
    X86Reg8::DL => 0xda,
    _ => panic!("Cannot rotate register"),
  };
  2
}

fn emit_rotate_right(register: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0xd0;
  exec[1] = match register {
    X86Reg8::AH => 0xcc,
    X86Reg8::BH => 0xcf,
    X86Reg8::BL => 0xcb,
    X86Reg8::CH => 0xcd,
    X86Reg8::CL => 0xc9,
    X86Reg8::DH => 0xce,
    X86Reg8::DL => 0xca,
    _ => panic!("Cannot rotate register"),
  };
  2
}

fn emit_shift_left(register: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0xd0; // sal register
  exec[1] = match register {
    X86Reg8::AH => 0xe4,
    X86Reg8::BH => 0xe7,
    X86Reg8::BL => 0xe3,
    X86Reg8::CH => 0xe5,
    X86Reg8::CL => 0xe1,
    X86Reg8::DH => 0xe6,
    X86Reg8::DL => 0xe2,
    _ => panic!("Cannot rotate register"),
  };
  2
}

fn emit_shift_right(register: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0xd0; // sar register
  exec[1] = match register {
    X86Reg8::AH => 0xfc,
    X86Reg8::BH => 0xff,
    X86Reg8::BL => 0xfb,
    X86Reg8::CH => 0xfd,
    X86Reg8::CL => 0xf9,
    X86Reg8::DH => 0xfe,
    X86Reg8::DL => 0xfa,
    _ => panic!("Cannot rotate register"),
  };
  2
}

fn emit_shift_right_logical(register: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0xd0; // shr register
  exec[1] = match register {
    X86Reg8::AH => 0xec,
    X86Reg8::BH => 0xef,
    X86Reg8::BL => 0xeb,
    X86Reg8::CH => 0xed,
    X86Reg8::CL => 0xe9,
    X86Reg8::DH => 0xee,
    X86Reg8::DL => 0xea,
    _ => panic!("Cannot rotate register"),
  };
  2
}

fn emit_swap(register: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0xc0; // rol reg, 4
  exec[1] = match register {
    X86Reg8::AH => 0xc4,
    X86Reg8::BH => 0xc7,
    X86Reg8::BL => 0xc3,
    X86Reg8::CH => 0xc5,
    X86Reg8::CL => 0xc1,
    X86Reg8::DH => 0xc6,
    X86Reg8::DL => 0xc2,
    _ => panic!("Cannot rotate register"),
  };
  exec[2] = 4;
  3
}

fn emit_complement_a(exec: &mut [u8]) -> usize {
  exec[0] = 0xf6; // NOT AH
  exec[1] = 0xd4;
  2
}

fn emit_complement_carry(exec: &mut [u8]) -> usize {
  exec[0] = 0x34; // XOR AL, 0x10
  exec[1] = 0x10;
  2
}

fn emit_jump(addr: u16, exec: &mut [u8]) -> usize {
  // A jump will cause the block to end
  // To perform the jump, simply update the IP register (r9)
  // The emulator will stop writing instructions at a jump, and the epilogue
  // will return
  exec[0] = 0x66;
  exec[1] = 0x41;
  exec[2] = 0xb9;
  emit_immediate_u16(addr, &mut exec[3..]);
  5
}

fn emit_jump_hl(exec: &mut [u8]) -> usize {
  // shortcut for "mov r9w, cx"
  exec[0] = 0x66;
  exec[1] = 0x41;
  exec[2] = 0x89;
  exec[3] = 0xc9;
  4
}

fn emit_flag_test(test: u8, exec: &mut [u8]) -> usize {
  // test al, value
  exec[0] = 0xa8;
  exec[1] = test;
  2
}

fn emit_bit_test(reg: X86Reg8, mask: u8, exec: &mut [u8]) -> usize {
  let reg_to_test = match reg {
    X86Reg8::AH => 0xc4,
    X86Reg8::BH => 0xc7,
    X86Reg8::BL => 0xc3,
    X86Reg8::CH => 0xc5,
    X86Reg8::CL => 0xc1,
    X86Reg8::DH => 0xc6,
    X86Reg8::DL => 0xc2,
    _ => panic!("Cannot test register"),
  };
  let code = [
    0xf6, reg_to_test, mask, // test reg, mask
    0x41, 0x0f, 0x94, 0xc6, // setz sil
    0x41, 0xd0, 0xce, // ror sil
    0x24, 0x00, // and al, 0x00 ; clear negative flag
    0x0c, 0x20, // or al, 0x20 ; and set half-carry flag?
    0x44, 0x08, 0xf0, // or al, sil
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  length
}

fn emit_jump_nonzero(relative: i32, exec: &mut [u8]) -> usize {
  exec[0] = 0x75;
  exec[1] = relative as u8;
  2
}

fn emit_jump_zero(relative: i8, exec: &mut [u8]) -> usize {
  exec[0] = 0x74;
  exec[1] = relative as u8;
  2
}

fn emit_ip_increment(amount: usize, exec: &mut [u8]) -> usize {
  /*exec[0] = 0x66;
  exec[1] = 0x41;
  exec[2] = 0x83;
  exec[3] = 0xc5;
  exec[4] = amount as u8;
  5
  */
  exec[0] = 0x49; // add r9, amount
  exec[1] = 0x83;
  exec[2] = 0xc1;
  exec[3] = amount as u8;
  4
}

fn emit_ip_signed_offset(offset: i8, exec: &mut [u8]) -> usize {
  exec[0] = 0x66; // add r9w, offset
  exec[1] = 0x41;
  exec[2] = 0x83;
  exec[3] = 0xc1;
  exec[4] = offset as u8;
  5
}

fn emit_sp_signed_offset(offset: i8, exec: &mut [u8]) -> usize {
  exec[0] = 0x66; // add r8w, offset
  exec[1] = 0x41;
  exec[2] = 0x83;
  exec[3] = 0xc0;
  exec[4] = offset as u8;
  5
}

fn emit_register_or(reg: X86Reg8, mask: u8, exec: &mut [u8]) -> usize {
  exec[0] = 0x80;
  exec[1] = match reg {
    X86Reg8::AH => 0xcc,
    X86Reg8::BH => 0xcf,
    X86Reg8::BL => 0xcb,
    X86Reg8::CH => 0xcd,
    X86Reg8::CL => 0xc9,
    X86Reg8::DH => 0xce,
    X86Reg8::DL => 0xca,
    _ => panic!("Cannot or register"),
  };
  exec[2] = mask;
  3
}

fn emit_register_and(reg: X86Reg8, mask: u8, exec: &mut [u8]) -> usize {
  exec[0] = 0x80;
  exec[1] = match reg {
    X86Reg8::AH => 0xe4,
    X86Reg8::BH => 0xe7,
    X86Reg8::BL => 0xe3,
    X86Reg8::CH => 0xe5,
    X86Reg8::CL => 0xe1,
    X86Reg8::DH => 0xe6,
    X86Reg8::DL => 0xe2,
    _ => panic!("Cannot and register"),
  };
  exec[2] = mask;
  3
}

/// The infamous DAA, used to adjust BCD math
fn emit_daa(exec: &mut [u8]) -> usize {
  let code = [
    0x66, 0x50, // push ax
    0x89, 0xc6, // mov esi, eax
    0x81, 0xe6, 0x00, 0x0f, 0x00, 0x00, // and esi, 0x0f00
    0x81, 0xfe, 0x00, 0x09, 0x00, 0x00, // cmp esi, 0x0900
    0x7f, 0x0e, // jg fix_low
    0x89, 0xc6, // mov esi, eax
    0x83, 0xe6, 0x20, // and esi, 0x20
    0x75, 0x07, // jne fix_low
    0x25, 0xd0, 0xff, 0x00, 0x00, // and eax, 0xffd0
    0xeb, 0x1d, // jmp daa_continue

    // fix_low:
    0x0f, 0xba, 0xe0, 0x06, // bt eax, 6
    0x72, 0x07, // jc +7
    0x05, 0x00, 0x06, 0x00, 0x00, // add eax, 0x0600
    0xeb, 0x05, // jmp +5
    0x2d, 0x00, 0x06, 0x00, 0x00, // sub eax, 0x0600
    0x9c, // pushfq
    0x5e, // pop rsi
    0x48, 0x83, 0xe6, 0x01, // and rsi, 1
    0xc1, 0xe6, 0x04, // shl esi, 4
    0x09, 0xf0, // or eax, esi

    // daa_continue:
    0x66, 0x5e, // pop si
    0x0f, 0xba, 0xe6, 0x04, // bt esi, 4
    0x72, 0x15, // jc fix_high
    0x81, 0xe6, 0x00, 0xff, 0x00, 0x00, // and esi, 0xff00
    0x81, 0xfe, 0x00, 0x99, 0x00, 0x00, // cmp esi, 0x9900
    0x7f, 0x07, // jg fix_high
    0x25, 0xe0, 0xff, 0x00, 0x00, // and eax, 0xffe0
    0xeb, 0x1a, // jmp done

    // fix_high:
    0x0f, 0xba, 0xe0, 0x06, // bt eax, 6
    0x72, 0x07, // jc +7
    0x05, 0x00, 0x60, 0x00, 0x00, // add eax, 0x6000
    0xeb, 0x05, // jmp +5
    0x2d, 0x00, 0x60, 0x00, 0x00, // sub eax, 0x6000
    0x25, 0xff, 0xff, 0x00, 0x00, // and eax, 0xffff
    0x83, 0xc8, 0x10, // or eax, 0x10

    // done:
    0x25, 0xd0, 0xff, 0x00, 0x00, // and eax, 0xffd0
    0x08, 0xe4, // or ah, ah
    0x75, 0x02, // jnz +2
    0x0c, 0x80, // or al, 0x80
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  length
}

fn emit_load_to_sp(exec: &mut [u8]) -> usize {
  exec[0] = 0x66; // mov r8w, cx
  exec[1] = 0x41;
  exec[2] = 0x89;
  exec[3] = 0xc8;
  4
}

fn emit_load_stack_offset(offset: i8, exec: &mut [u8]) -> usize {
  let code = [
    0x66, 0x44, 0x89, 0xc1, // mov cx, r8w
    0x66, 0x83, 0xc1, offset as u8, // add cx, offset
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  length
}

fn emit_memory_read(exec: &mut [u8], memory_base: usize, indirect_address: X86Reg16, dest_register: X86Reg8) -> usize {
  let fn_pointer = address_as_bytes(crate::mem::memory_read_byte as u64);
  let address_source = match indirect_address {
    X86Reg16::BX => 0xde,
    X86Reg16::CX => 0xce,
    X86Reg16::DX => 0xd6,
    _ => panic!("cannot read from address at register"),
  };
  let stack_offset = match dest_register {
    X86Reg8::BL => 0,
    X86Reg8::BH => 1,
    X86Reg8::DL => 8,
    X86Reg8::DH => 9,
    X86Reg8::CL => 16,
    X86Reg8::CH => 17,
    X86Reg8::AL => 24,
    X86Reg8::AH => 25,
  };
  let memory_pointer = address_as_bytes(memory_base as u64);
  let code = [
    0x50, // push rax
    0x51, // push rcx
    0x52, // push rdx
    0x53, // push rbx
    0x48, 0x89, address_source, // mov rsi, indirect_address
    0x48, 0xbf, // movabs rdi, memory_pointer
      memory_pointer[0],
      memory_pointer[1],
      memory_pointer[2],
      memory_pointer[3],
      memory_pointer[4],
      memory_pointer[5],
      memory_pointer[6],
      memory_pointer[7],

    0x48, 0xb8, // movabs rax, fn_pointer
      fn_pointer[0],
      fn_pointer[1],
      fn_pointer[2],
      fn_pointer[3],
      fn_pointer[4],
      fn_pointer[5],
      fn_pointer[6],
      fn_pointer[7],
    0xff, 0xd0, // call rax
    // $rax will hold the return result
    // move the value from $al to the appropriate register on the stack before
    // popping all of them
    0x88, 0x44, 0x24, stack_offset, // mov [rsp + stack_offset], al
    0x5b, // pop rbx
    0x5a, // pop rdx
    0x59, // pop rcx
    0x58, // pop rax
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  length
}

fn emit_memory_write(exec: &mut [u8], memory_base: usize, indirect_address: X86Reg16, source: X86Reg8) -> usize {
  let fn_pointer = address_as_bytes(crate::mem::memory_write_byte as u64);
  let address_dest = match indirect_address {
    X86Reg16::BX => 0xde,
    X86Reg16::CX => 0xce,
    X86Reg16::DX => 0xd6,
    _ => panic!("cannot read from address at register"),
  };
  let memory_pointer = address_as_bytes(memory_base as u64);
  let code = [
    0x50, // push rax
    0x51, // push rcx
    0x52, // push rdx
    0x48, 0x89, address_dest, // mov rsi, indirect_address
    0x48, 0xbf, // movabs rdi, memory_pointer
      memory_pointer[0],
      memory_pointer[1],
      memory_pointer[2],
      memory_pointer[3],
      memory_pointer[4],
      memory_pointer[5],
      memory_pointer[6],
      memory_pointer[7],
    0x88, register_to_register(source, X86Reg8::DL), // mov dl, source
    0x48, 0x81, 0xe2, 0xff, 0x00, 0x00, 0x00, // and rdx, 0xff

    0x48, 0xb8, // movabs rax, fn_pointer
      fn_pointer[0],
      fn_pointer[1],
      fn_pointer[2],
      fn_pointer[3],
      fn_pointer[4],
      fn_pointer[5],
      fn_pointer[6],
      fn_pointer[7],
    0xff, 0xd0, // call rax
    0x5a, // pop rdx
    0x59, // pop rcx
    0x58, // pop rax
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  length
}

fn emit_memory_write_literal(exec: &mut [u8], memory_base: usize, indirect_address: X86Reg16, value: u8) -> usize {
  let fn_pointer = address_as_bytes(crate::mem::memory_write_byte as u64);
  let address_dest = match indirect_address {
    X86Reg16::BX => 0xde,
    X86Reg16::CX => 0xce,
    X86Reg16::DX => 0xd6,
    _ => panic!("cannot read from address at register"),
  };
  let memory_pointer = address_as_bytes(memory_base as u64);
  let code = [
    0x50, // push rax
    0x51, // push rcx
    0x52, // push rdx
    0x48, 0x89, address_dest, // mov rsi, indirect_address
    0x48, 0xbf, // movabs rdi, memory_pointer
      memory_pointer[0],
      memory_pointer[1],
      memory_pointer[2],
      memory_pointer[3],
      memory_pointer[4],
      memory_pointer[5],
      memory_pointer[6],
      memory_pointer[7],
    0xb2, value, // mov dl, value
    0x48, 0x81, 0xe2, 0xff, 0x00, 0x00, 0x00, // and rdx, 0xff

    0x48, 0xb8, // movabs rax, fn_pointer
      fn_pointer[0],
      fn_pointer[1],
      fn_pointer[2],
      fn_pointer[3],
      fn_pointer[4],
      fn_pointer[5],
      fn_pointer[6],
      fn_pointer[7],
    0xff, 0xd0, // call rax
    0x5a, // pop rdx
    0x59, // pop rcx
    0x58, // pop rax
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  length
}

fn emit_write_stack_to_memory(exec: &mut [u8], memory_base: usize, address: u16) -> usize {
  let fn_pointer = address_as_bytes(crate::mem::memory_write_word as u64);
  let memory_pointer = address_as_bytes(memory_base as u64);
  let code = [
    0x50, // push rax
    0x51, // push rcx
    0x52, // push rdx
    0x66, 0xbe, (address & 0xff) as u8, (address >> 8) as u8, // mov si, address
    0x48, 0xbf, // movabs rdi, memory_pointer
      memory_pointer[0],
      memory_pointer[1],
      memory_pointer[2],
      memory_pointer[3],
      memory_pointer[4],
      memory_pointer[5],
      memory_pointer[6],
      memory_pointer[7],
    0x4c, 0x89, 0xc2, // mov rdx, r8

    0x48, 0xb8, // movabs rax, fn_pointer
      fn_pointer[0],
      fn_pointer[1],
      fn_pointer[2],
      fn_pointer[3],
      fn_pointer[4],
      fn_pointer[5],
      fn_pointer[6],
      fn_pointer[7],
    0xff, 0xd0, // call rax
    0x5a, // pop rdx
    0x59, // pop rcx
    0x58, // pop rax
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  length
}

fn emit_write_a_to_memory(exec: &mut [u8], memory_base: usize, address: u16) -> usize {
  let fn_pointer = address_as_bytes(crate::mem::memory_write_byte as u64);
  let memory_pointer = address_as_bytes(memory_base as u64);
  let code = [
    0x50, // push rax
    0x51, // push rcx
    0x52, // push rdx
    0x66, 0xbe, (address & 0xff) as u8, (address >> 8) as u8, // mov si, address
    0x48, 0xbf, // movabs rdi, memory_pointer
      memory_pointer[0],
      memory_pointer[1],
      memory_pointer[2],
      memory_pointer[3],
      memory_pointer[4],
      memory_pointer[5],
      memory_pointer[6],
      memory_pointer[7],
    0x88, 0xe2, // mov dl, ah

    0x48, 0xb8, // movabs rax, fn_pointer
      fn_pointer[0],
      fn_pointer[1],
      fn_pointer[2],
      fn_pointer[3],
      fn_pointer[4],
      fn_pointer[5],
      fn_pointer[6],
      fn_pointer[7],
    0xff, 0xd0, // call rax
    0x5a, // pop rdx
    0x59, // pop rcx
    0x58, // pop rax
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  length
}

fn emit_read_a_from_memory(exec: &mut [u8], memory_base: usize, address: u16) -> usize {
  let fn_pointer = address_as_bytes(crate::mem::memory_read_byte as u64);
  let memory_pointer = address_as_bytes(memory_base as u64);
  let code = [
    0x50, // push rax
    0x51, // push rcx
    0x52, // push rdx
    0x66, 0xbe, (address & 0xff) as u8, (address >> 8) as u8, // mov si, address
    0x48, 0xbf, // movabs rdi, memory_pointer
      memory_pointer[0],
      memory_pointer[1],
      memory_pointer[2],
      memory_pointer[3],
      memory_pointer[4],
      memory_pointer[5],
      memory_pointer[6],
      memory_pointer[7],

    0x48, 0xb8, // movabs rax, fn_pointer
      fn_pointer[0],
      fn_pointer[1],
      fn_pointer[2],
      fn_pointer[3],
      fn_pointer[4],
      fn_pointer[5],
      fn_pointer[6],
      fn_pointer[7],
    0xff, 0xd0, // call rax
    0x88, 0x44, 0x24, 0x11, // mov [rsp + 17], al
    0x5a, // pop rdx
    0x59, // pop rcx
    0x58, // pop rax
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  length
}

fn emit_load_to_high_mem(exec: &mut [u8], memory_base: usize) -> usize {
  let fn_pointer = address_as_bytes(crate::mem::memory_write_byte as u64);
  let memory_pointer = address_as_bytes(memory_base as u64);
  let code = [
    0x50, // push rax
    0x51, // push rcx
    0x52, // push rdx
    0x66, 0x89, 0xde, // mov si, bx
    0x66, 0x81, 0xce, 0x00, 0xff, // or si, 0xff00
    0x48, 0xbf, // movabs rdi, memory_pointer
      memory_pointer[0],
      memory_pointer[1],
      memory_pointer[2],
      memory_pointer[3],
      memory_pointer[4],
      memory_pointer[5],
      memory_pointer[6],
      memory_pointer[7],
    0x88, 0xe2, // mov dl, ah

    0x48, 0xb8, // movabs rax, fn_pointer
      fn_pointer[0],
      fn_pointer[1],
      fn_pointer[2],
      fn_pointer[3],
      fn_pointer[4],
      fn_pointer[5],
      fn_pointer[6],
      fn_pointer[7],
    0xff, 0xd0, // call rax
    0x5a, // pop rdx
    0x59, // pop rcx
    0x58, // pop rax
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  length
}

fn emit_load_from_high_mem(exec: &mut [u8], memory_base: usize) -> usize {
  let fn_pointer = address_as_bytes(crate::mem::memory_read_byte as u64);
  let memory_pointer = address_as_bytes(memory_base as u64);
  let code = [
    0x50, // push rax
    0x51, // push rcx
    0x52, // push rdx
    0x66, 0x89, 0xde, // mov si, bx
    0x66, 0x81, 0xce, 0x00, 0xff, // or si, 0xff00
    0x48, 0xbf, // movabs rdi, memory_pointer
      memory_pointer[0],
      memory_pointer[1],
      memory_pointer[2],
      memory_pointer[3],
      memory_pointer[4],
      memory_pointer[5],
      memory_pointer[6],
      memory_pointer[7],

    0x48, 0xb8, // movabs rax, fn_pointer
      fn_pointer[0],
      fn_pointer[1],
      fn_pointer[2],
      fn_pointer[3],
      fn_pointer[4],
      fn_pointer[5],
      fn_pointer[6],
      fn_pointer[7],
    0xff, 0xd0, // call rax
    0x88, 0x44, 0x24, 0x11, // mov [rsp + 17], al
    0x5a, // pop rdx
    0x59, // pop rcx
    0x58, // pop rax
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  length
}

fn emit_push(source: X86Reg16, memory_base: usize, exec: &mut [u8]) -> usize {
  let fn_pointer = address_as_bytes(crate::mem::memory_write_word as u64);
  let memory_pointer = address_as_bytes(memory_base as u64);
  let load_source_bytes = match source {
    X86Reg16::AX => (0x89, 0xc2, 0x90),
    X86Reg16::BX => (0x89, 0xda, 0x90),
    X86Reg16::CX => (0x89, 0xca, 0x90),
    X86Reg16::DX => (0x89, 0xd2, 0x90),
    X86Reg16::R9 => (0x44, 0x89, 0xca),
    _ => unreachable!("Unsupported push register"),
  };
  let code = [
    0x50, // push rax
    0x51, // push rcx
    0x52, // push rdx
    0x49, 0x83, 0xe8, 0x02, // sub r8, 2
    0x49, 0x81, 0xe0, 0xff, 0xff, 0x00, 0x00, // and r8, 0xffff
    0x4c, 0x89, 0xc6, // mov rsi, r8
    0x48, 0xbf, // movabs rdi, memory_pointer
      memory_pointer[0],
      memory_pointer[1],
      memory_pointer[2],
      memory_pointer[3],
      memory_pointer[4],
      memory_pointer[5],
      memory_pointer[6],
      memory_pointer[7],
    0x66, load_source_bytes.0, load_source_bytes.1, load_source_bytes.2, // mov dx, source
    0x48, 0x81, 0xe2, 0xff, 0xff, 0x00, 0x00, // and rdx, 0xffff

    0x48, 0xb8, // movabs rax, fn_pointer
      fn_pointer[0],
      fn_pointer[1],
      fn_pointer[2],
      fn_pointer[3],
      fn_pointer[4],
      fn_pointer[5],
      fn_pointer[6],
      fn_pointer[7],
    0xff, 0xd0, // call rax
    0x5a, // pop rdx
    0x59, // pop rcx
    0x58, // pop rax
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  length
}

fn emit_pop(dest: X86Reg16, memory_base: usize, exec: &mut [u8]) -> usize {
  let fn_pointer = address_as_bytes(crate::mem::memory_read_word as u64);
  let memory_pointer = address_as_bytes(memory_base as u64);
  let stack_offset = match dest {
    X86Reg16::AX => 32,
    X86Reg16::BX => 8,
    X86Reg16::CX => 24,
    X86Reg16::DX => 16,
    X86Reg16::R9 => 0,
    _ => unreachable!("Unsupported push register"),
  };
  let code = [
    0x50, // push rax
    0x51, // push rcx
    0x52, // push rdx
    0x53, // push rbx
    0x41, 0x51, // push r9
    0x4c, 0x89, 0xc6, // mov rsi, r8
    0x49, 0x83, 0xc0, 0x02, // add r8, 2
    0x49, 0x81, 0xe0, 0xff, 0xff, 0x00, 0x00, // and r8, 0xffff
    0x48, 0xbf, // movabs rdi, memory_pointer
      memory_pointer[0],
      memory_pointer[1],
      memory_pointer[2],
      memory_pointer[3],
      memory_pointer[4],
      memory_pointer[5],
      memory_pointer[6],
      memory_pointer[7],
    0x48, 0xb8, // movabs rax, fn_pointer
      fn_pointer[0],
      fn_pointer[1],
      fn_pointer[2],
      fn_pointer[3],
      fn_pointer[4],
      fn_pointer[5],
      fn_pointer[6],
      fn_pointer[7],
    0xff, 0xd0, // call rax
    0x66, 0x89, 0x44, 0x24, stack_offset, // mov [rsp + stack_offset], ax

    0x41, 0x59, // pop r9
    0x5b, // pop rbx
    0x5a, // pop rdx
    0x59, // pop rcx
    0x58, // pop rax
  ];
  let mut length = code.len();
  exec[..length].copy_from_slice(&code);
  if let X86Reg16::AX = dest {
    // need to clear the lower 4 bits of al
    // and al, 0xf0
    exec[length] = 0x24;
    length += 1;
    exec[length] = 0xf0;
    length += 1;
  }
  length
}

fn emit_hl_indirect_partial_read(memory_base: usize, exec: &mut [u8]) -> usize {
  let fn_pointer = address_as_bytes(crate::mem::memory_read_byte as u64);
  let memory_pointer = address_as_bytes(memory_base as u64);
  let code = [
    0x50, // push rax
    0x51, // push rcx
    0x52, // push rdx
    0x48, 0x89, 0xce, // mov rsi, rcx
    0x48, 0xbf, // movabs rdi, memory_pointer
      memory_pointer[0],
      memory_pointer[1],
      memory_pointer[2],
      memory_pointer[3],
      memory_pointer[4],
      memory_pointer[5],
      memory_pointer[6],
      memory_pointer[7],

    0x48, 0xb8, // movabs rax, fn_pointer
      fn_pointer[0],
      fn_pointer[1],
      fn_pointer[2],
      fn_pointer[3],
      fn_pointer[4],
      fn_pointer[5],
      fn_pointer[6],
      fn_pointer[7],
    0xff, 0xd0, // call rax
    0x48, 0x89, 0xc2, // mov rdx, rax
    0x66, 0x8b, 0x44, 0x24, 0x10, // mov ax, [rsp + 16]
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  length
}

fn emit_hl_indirect_partial_write(memory_base: usize, exec: &mut [u8]) -> usize {
  let fn_pointer = address_as_bytes(crate::mem::memory_write_byte as u64);
  let memory_pointer = address_as_bytes(memory_base as u64);
  let code = [
    0x88, 0x44, 0x24, 0x10, // mov [rsp + 16], al
    0x48, 0x8b, 0x74, 0x24, 0x08, // mov rsi, [rsp + 8]
    0x48, 0xbf, // movabs rdi, memory_pointer
      memory_pointer[0],
      memory_pointer[1],
      memory_pointer[2],
      memory_pointer[3],
      memory_pointer[4],
      memory_pointer[5],
      memory_pointer[6],
      memory_pointer[7],

    0x48, 0xb8, // movabs rax, fn_pointer
      fn_pointer[0],
      fn_pointer[1],
      fn_pointer[2],
      fn_pointer[3],
      fn_pointer[4],
      fn_pointer[5],
      fn_pointer[6],
      fn_pointer[7],
    0xff, 0xd0, // call rax
    0x5a, // pop rdx
    0x59, // pop rcx
    0x58, // pop rax
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  length
}

/// Read the value stored at (HL) into E
/// Make sure $rdx can be restored after this result is used
fn emit_hl_indirect_read(memory_base: usize, exec: &mut [u8]) -> usize {
  let fn_pointer = address_as_bytes(crate::mem::memory_read_byte as u64);
  let memory_pointer = address_as_bytes(memory_base as u64);
  let code = [
    0x50, // push rax
    0x51, // push rcx
    0x48, 0x89, 0xce, // mov rsi, rcx
    0x48, 0xbf, // movabs rdi, memory_pointer
      memory_pointer[0],
      memory_pointer[1],
      memory_pointer[2],
      memory_pointer[3],
      memory_pointer[4],
      memory_pointer[5],
      memory_pointer[6],
      memory_pointer[7],

    0x48, 0xb8, // movabs rax, fn_pointer
      fn_pointer[0],
      fn_pointer[1],
      fn_pointer[2],
      fn_pointer[3],
      fn_pointer[4],
      fn_pointer[5],
      fn_pointer[6],
      fn_pointer[7],
    0xff, 0xd0, // call rax
    0x48, 0x89, 0xc2, // mov rdx, rax
    0x59, // pop rcx
    0x58, // pop rax
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  length
}

fn emit_push_register(reg: X86Reg64, exec: &mut [u8]) -> usize {
  match reg {
    X86Reg64::RAX => {
      exec[0] = 0x50;
      1
    },
    X86Reg64::RBX => {
      exec[0] = 0x53;
      1
    },
    X86Reg64::RCX => {
      exec[0] = 0x51;
      1
    },
    X86Reg64::RDX => {
      exec[0] = 0x52;
      1
    },
  }
}

fn emit_pop_register(reg: X86Reg64, exec: &mut [u8]) -> usize {
  match reg {
    X86Reg64::RAX => {
      exec[0] = 0x58;
      1
    },
    X86Reg64::RBX => {
      exec[0] = 0x5b;
      1
    },
    X86Reg64::RCX => {
      exec[0] = 0x59;
      1
    },
    X86Reg64::RDX => {
      exec[0] = 0x5a;
      1
    },
  }
}

fn address_as_bytes(addr: u64) -> [u8; 8] {
  [
    (addr & 0xff) as u8,
    ((addr >> 8) & 0xff) as u8,
    ((addr >> 16) & 0xff) as u8,
    ((addr >> 24) & 0xff) as u8,
    ((addr >> 32) & 0xff) as u8,
    ((addr >> 40) & 0xff) as u8,
    ((addr >> 48) & 0xff) as u8,
    ((addr >> 56) & 0xff) as u8,
  ]
}

fn register_to_register(src: X86Reg8, dest: X86Reg8) -> u8 {
  let reg = match src {
    X86Reg8::AL => 0,
    X86Reg8::CL => 0b001 << 3,
    X86Reg8::DL => 0b010 << 3,
    X86Reg8::BL => 0b011 << 3,
    X86Reg8::AH => 0b100 << 3,
    X86Reg8::CH => 0b101 << 3,
    X86Reg8::DH => 0b110 << 3,
    X86Reg8::BH => 0b111 << 3,
  };
  let rm = match dest {
    X86Reg8::AL => 0,
    X86Reg8::CL => 0b001,
    X86Reg8::DL => 0b010,
    X86Reg8::BL => 0b011,
    X86Reg8::AH => 0b100,
    X86Reg8::CH => 0b101,
    X86Reg8::DH => 0b110,
    X86Reg8::BH => 0b111,
  };
  0xc0 | reg | rm
}

fn map_register_16(gb_reg: Register16) -> X86Reg16 {
  match gb_reg {
    Register16::AF => X86Reg16::AX,
    Register16::BC => X86Reg16::BX,
    Register16::DE => X86Reg16::DX,
    Register16::HL => X86Reg16::CX,
    Register16::SP => X86Reg16::R8,
  }
}

fn map_register_8(gb_reg: Register8) -> X86Reg8 {
  match gb_reg {
    Register8::A => X86Reg8::AH,
    Register8::B => X86Reg8::BH,
    Register8::C => X86Reg8::BL,
    Register8::D => X86Reg8::DH,
    Register8::E => X86Reg8::DL,
    Register8::H => X86Reg8::CH,
    Register8::L => X86Reg8::CL,
  }
}

fn map_indirect_location_to_register(location: IndirectLocation) -> X86Reg16 {
  match location {
    IndirectLocation::BC => X86Reg16::BX,
    IndirectLocation::DE => X86Reg16::DX,
    IndirectLocation::HL
      | IndirectLocation::HLDecrement
      | IndirectLocation::HLIncrement => X86Reg16::CX,
  }
}

#[derive(Copy, Clone)]
enum X86Reg8 {
  AH,
  AL,
  BH,
  BL,
  CH,
  CL,
  DH,
  DL,
}

#[derive(Copy, Clone)]
enum X86Reg16 {
  AX,
  BX,
  CX,
  DX,
  R8,
  R9,
}

enum X86Reg64 {
  RAX,
  RBX,
  RCX,
  RDX,
}

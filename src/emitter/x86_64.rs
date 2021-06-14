use crate::decoder::ops::{Op, IndirectLocation, JumpCondition, Register8, Register16, Source8};

pub struct Emitter {
  
}

impl Emitter {
  pub fn new() -> Self {
    Self {

    }
  }

  pub fn encode_prelude(&self, exec: &mut [u8]) -> usize {
    let code = [
      // preserve scratch registers that will be modified
      0x53, // push rbx
      0x41, 0x54, // push r12
      0x41, 0x55, // push r13
      // begin method, load all registers from a struct in memory
      // the only argument (rdi) will be a pointer to the struct
      0x8b, 0x07, // mov eax, [rdi]
      0x8b, 0x5f, 0x04, // mov ebx, [rdi + 4]
      0x8b, 0x57, 0x08, // mov edx, [rdi + 8]
      0x8b, 0x4f, 0x0c, // mov ecx, [rdi + 12]
      0x66, 0x44, 0x8b, 0x67, 0x10, // mov r12w, [rdi + 16]
      0x66, 0x44, 0x8b, 0x6f, 0x14, // mov r13w, [rdi + 20]
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
      0x66, 0x44, 0x89, 0x67, 0x10, // mov [rdi + 16], r12w
      0x66, 0x44, 0x89, 0x6f, 0x14, // mov [rdi + 20], r13w
      // Restore scratch registers to their original value
      0x41, 0x5d, // pop r13
      0x41, 0x5c, // pop r12
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
      Op::Load8Immediate(reg, value) => self.encode_load_8(reg, value, ip_increment, exec),
      Op::Increment8(reg) => self.encode_increment_8(reg, ip_increment, exec),
      Op::Decrement8(reg) => self.encode_decrement_8(reg, ip_increment, exec),
      Op::Increment16(reg) => self.encode_increment_16(reg, ip_increment, exec),
      Op::Decrement16(reg) => self.encode_decrement_16(reg, ip_increment, exec),
      Op::Add8(dest, src) => self.encode_add_register_8(dest, src, ip_increment, exec),
      Op::AddWithCarry8(dest, src) => self.encode_add_register_8_with_carry(dest, src, ip_increment, exec),
      Op::AddHL(src) => self.encode_add_hl(src, ip_increment, exec),
      Op::AddAbsolute8(value) => self.encode_add_absolute_8(value, ip_increment, exec),
      Op::Sub8(dest, src) => self.encode_sub_register_8(dest, src, ip_increment, exec),
      Op::SubWithCarry8(dest, src) => self.encode_sub_register_8_with_carry(dest, src, ip_increment, exec),
      Op::And8(dest, src) => self.encode_and_register_8(dest, src, ip_increment, exec),
      Op::Xor8(dest, src) => self.encode_xor_register_8(dest, src, ip_increment, exec),
      Op::Or8(dest, src) => self.encode_or_register_8(dest, src, ip_increment, exec),
      Op::RotateLeftA => self.encode_rotate_left_a(ip_increment, exec),
      Op::RotateLeftCarryA => self.encode_rotate_left_carry_a(ip_increment, exec),
      Op::RotateLeft(reg) => self.encode_rotate_left(reg, ip_increment, exec),
      Op::RotateLeftCarry(reg) => self.encode_rotate_left_carry(reg, ip_increment, exec),
      Op::RotateRightA => self.encode_rotate_right_a(ip_increment, exec),
      Op::RotateRightCarryA => self.encode_rotate_right_carry_a(ip_increment, exec),
      Op::RotateRight(reg) => self.encode_rotate_right(reg, ip_increment, exec),
      Op::RotateRightCarry(reg) => self.encode_rotate_right_carry(reg, ip_increment, exec),
      Op::ComplementA => self.encode_complement_a(ip_increment, exec),
      Op::SetCarryFlag => self.encode_set_carry(ip_increment, exec),
      Op::ComplementCarryFlag => self.encode_complement_carry(ip_increment, exec),

      Op::Jump(cond, address) => self.encode_jump(cond, address, exec),

      Op::Invalid(code) => panic!("Invalid OP: {:#04x}", code),
      _ => panic!("unsupported op"),
    }
  }

  pub fn encode_noop(&self, exec: &mut [u8]) -> usize {
    emit_ip_increment(1, exec)
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

  pub fn encode_sub_register_8(&self, dest: Register8, src: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    0
  }

  pub fn encode_sub_register_8_with_carry(&self, dest: Register8, src: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    0
  }

  pub fn encode_and_register_8(&self, dest: Register8, src: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_and_register_8(map_register_8(dest), map_register_8(src), exec);
    len += emit_store_flags(0x80, false, &mut exec[len..]);
    len += emit_force_flags_off(0x05, &mut exec[len..]);
    len += emit_force_flags_on(0x20, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_or_register_8(&self, dest: Register8, src: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_or_register_8(map_register_8(dest), map_register_8(src), exec);
    len += emit_store_flags(0x80, false, &mut exec[len..]);
    len += emit_force_flags_off(0x07, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_xor_register_8(&self, dest: Register8, src: Register8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_xor_register_8(map_register_8(dest), map_register_8(src), exec);
    len += emit_store_flags(0x80, false, &mut exec[len..]);
    len += emit_force_flags_off(0x07, &mut exec[len..]);
    len + emit_ip_increment(ip_increment, &mut exec[len..])
  }

  pub fn encode_add_absolute_8(&self, value: u8, ip_increment: usize, exec: &mut [u8]) -> usize {
    let mut len = emit_add_absolute_8(value, exec);
    len += emit_store_flags(0xf0, false, &mut exec[len..]);
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

  pub fn encode_load_to_indirect(&self, _location: IndirectLocation, _value: Source8, _ip_increment: usize, _exec: &mut [u8]) -> usize {
    panic!("unimplemented");
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
        len += emit_move_16(X86Reg16::R13, address, &mut exec[len..]);
      },
      JumpCondition::NonZero => {
        len = emit_ip_increment(3, exec);
        len += emit_flag_test(0x80, &mut exec[len..]);
        len += emit_jump_nonzero(5, &mut exec[len..]);
        len += emit_move_16(X86Reg16::R13, address, &mut exec[len..]);
      },
      JumpCondition::Carry => {
        len = emit_ip_increment(3, exec);
        len += emit_flag_test(0x10, &mut exec[len..]);
        len += emit_jump_zero(5, &mut exec[len..]);
        len += emit_move_16(X86Reg16::R13, address, &mut exec[len..]);
      },
      JumpCondition::NoCarry => {
        len = emit_ip_increment(3, exec);
        len += emit_flag_test(0x10, &mut exec[len..]);
        len += emit_jump_nonzero(5, &mut exec[len..]);
        len += emit_move_16(X86Reg16::R13, address, &mut exec[len..]);
      },
    }
    len
  }
}

fn emit_immediate_u16(value: u16, exec: &mut [u8]) {
  exec[0] = (value & 0xff) as u8;
  exec[1] = (value >> 8) as u8;
}

fn emit_move_16(dest: X86Reg16, value: u16, exec: &mut [u8]) -> usize {
  exec[0] = 0x66;
  let mut pointer = 1;
  match dest {
    X86Reg16::AX => exec[pointer] = 0xb8,
    X86Reg16::CX => exec[pointer] = 0xb9,
    X86Reg16::DX => exec[pointer] = 0xba,
    X86Reg16::BX => exec[pointer] = 0xbb,
    X86Reg16::R12 => {
      exec[pointer] = 0x41;
      pointer += 1;
      exec[pointer] = 0xbc;
    },
    X86Reg16::R13 => {
      exec[pointer] = 0x41;
      pointer += 1;
      exec[pointer] = 0xbd;
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
    X86Reg16::R12 => {
      // r12 is messier to deal with because we can't touch bits 8-15 directly
      let code = [
        0x44, 0x00, 0xe1, // add cl, r12b
        0x9c, // pushf
        0xc1, 0xc9, 0x08, // ror ecx, 8
        0x41, 0xc1, 0xcc, 0x08, // ror r12d, 8
        0x9d, // popf
        0x44, 0x10, 0xe1, // adc cl, r12b
        0x9c, // pushf
        0xc1, 0xc1, 0x08, // rol ecx, 8
        0x41, 0xc1, 0xc4, 0x08, // rol r12d, 8
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
    0x41, 0x0f, 0x94, 0xc0, // setz r8b
    0x41, 0xd0, 0xc8, // ror r8b
    0x44, 0x08, 0xc0, // or al, r8b
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

fn emit_restore_carry(exec: &mut [u8]) -> usize {
  let code = [
    0x41, 0x88, 0xc0, // mov r8b, al
    0x24, 0x10, // and al, 0x10 ; isolate the carry bit
    0x04, 0xf0, // add al, 0xf0 ; if bit 4 is set, this will cause a carry
  ];
  let length = code.len();
  exec[..length].copy_from_slice(&code);
  length
}

fn emit_increment_8(dest: X86Reg8, exec: &mut [u8]) -> usize {
  exec[0] = 0xfe;
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
  // To perform the jump, simply update the IP register (r13)
  // The emulator will stop writing instructions at a jump, and the epilogue
  // will return
  exec[0] = 0x66;
  exec[1] = 0x41;
  exec[2] = 0xbd;
  emit_immediate_u16(addr, &mut exec[3..]);
  5
}

fn emit_flag_test(test: u8, exec: &mut [u8]) -> usize {
  // test al, value
  exec[0] = 0xa8;
  exec[1] = test;
  2
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
  exec[0] = 0x66;
  exec[1] = 0x41;
  exec[2] = 0x83;
  exec[3] = 0xc5;
  exec[4] = amount as u8;
  5
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
    Register16::SP => X86Reg16::R12,
  }
}

fn map_register_8(gb_reg: Register8) -> X86Reg8 {
  match gb_reg {
    Register8::A => X86Reg8::AH,
    Register8::F => X86Reg8::AL,
    Register8::B => X86Reg8::BH,
    Register8::C => X86Reg8::BL,
    Register8::D => X86Reg8::DH,
    Register8::E => X86Reg8::DL,
    Register8::H => X86Reg8::CH,
    Register8::L => X86Reg8::CL,
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
  R12,
  R13,
}

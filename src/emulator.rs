use crate::cache::CodeCache;
use crate::cpu::Registers;
use crate::mem::MemoryAreas;

pub struct Core {
  pub cache: CodeCache,
  pub registers: Registers,

  pub memory: MemoryAreas,
}

impl Core {
  pub fn with_code_block(code: Box<[u8]>) -> Self {
    Self {
      cache: CodeCache::new(),
      registers: Registers::new(),
      memory: MemoryAreas::with_rom(code),
    }
  }

  /// Run the next code block, then check for interrupts
  pub fn run_code_block(&mut self) {
    let address = {
      let ip = self.registers.ip as usize;
      let found_address = self.cache.get_address_for_ip(ip);
      if let Some(addr) = found_address {
        addr
      } else {
        self.cache.translate_code_block(&self.memory.rom, ip, self.memory.as_ptr())
      }
    };
    self.cache.call(address, &mut self.registers);
    println!("{:?}", self.registers);
  }
}

#[cfg(test)]
mod tests {
  use super::Core;

  #[test]
  fn load_8_bit_absolute() {
    let code = vec![
      0x3e, 0xa0, // LD A, 0xa0
      0x06, 0xb0, // LD B, 0xb0
      0x0e, 0xc0, // LD C, 0xc0
      0x16, 0xd0, // LD D, 0xd0
      0x1e, 0xe0, // LD E, 0xe0
      0x26, 0x11, // LD H, 0x11
      0x2e, 0x22, // LD L, 0x22
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xa000);
    assert_eq!(core.registers.get_bc(), 0xb0c0);
    assert_eq!(core.registers.get_de(), 0xd0e0);
    assert_eq!(core.registers.get_hl(), 0x1122);
    assert_eq!(core.registers.get_ip(), 14);
  }

  #[test]
  fn load_16_bit_absolute() {
    let code = vec![
      0x01, 0x22, 0x11, // LD BC, 0x1122
      0x11, 0x44, 0x33, // LD DE, 0x3344
      0x21, 0x66, 0x55, // LD HL, 0x5566
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_bc(), 0x1122);
    assert_eq!(core.registers.get_de(), 0x3344);
    assert_eq!(core.registers.get_hl(), 0x5566);
    assert_eq!(core.registers.get_ip(), 9);
  }

  #[test]
  fn increment_16_bit() {
    let code = vec![
      0x03, // INC BC
      0x13, // INC DE
      0x13, // INC DE
      0x23, // INC HL
      0x23, // INC HL
      0x23, // INC HL
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_bc(), 1);
    assert_eq!(core.registers.get_de(), 2);
    assert_eq!(core.registers.get_hl(), 3);
    assert_eq!(core.registers.get_ip(), 6);
  }

  #[test]
  fn decrement_16_bit() {
    let code = vec![
      0x01, 0x05, 0x00, // LD BC, 5
      0x11, 0x04, 0x00, // LD DE, 4
      0x21, 0x08, 0x00, // LD HL, 8
      0x0b, // DEC BC
      0x1b, // DEC DE
      0x1b, // DEC DE
      0x2b, // DEC HL
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_bc(), 4);
    assert_eq!(core.registers.get_de(), 2);
    assert_eq!(core.registers.get_hl(), 7);
    assert_eq!(core.registers.get_ip(), 13);
  }

  #[test]
  fn increment_8_bit() {
    let code = vec![
      0x3c, // INC A
      0x04, // INC B
      0x04, // INC B
      0x0c, // INC C
      0x0c, // INC C
      0x0c, // INC C
      0x14, // INC D
      0x14, // INC D
      0x14, // INC D
      0x14, // INC D
      0x1c, // INC E
      0x1c, // INC E
      0x1c, // INC E
      0x1c, // INC E
      0x1c, // INC E
      0x24, // INC H
      0x24, // INC H
      0x24, // INC H
      0x24, // INC H
      0x24, // INC H
      0x24, // INC H
      0x2c, // INC L
      0x2c, // INC L
      0x2c, // INC L
      0x2c, // INC L
      0x2c, // INC L
      0x2c, // INC L
      0x2c, // INC L
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af() & 0xff00, 0x0100);
    assert_eq!(core.registers.get_bc(), 0x0203);
    assert_eq!(core.registers.get_de(), 0x0405);
    assert_eq!(core.registers.get_hl(), 0x0607);
    assert_eq!(core.registers.get_ip(), 28);
  }

  #[test]
  fn increment_8_bit_flags() {
    { // test negative flag cleared
      let code = vec![
        0x3c, // INC A
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.registers.af = 0x0040; // negative flag set
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x0100);
    }
    { // test half-carry set
      let code = vec![
        0x3e, 0x0f, // LD A, 0x0f
        0x3c, // INC A
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x1020);
    }
    { // test zero flag set
      let code = vec![
        0x3e, 0xff, // LD A, 0xff
        0x3c, // INC A
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x00a0);
    }
    { // test carry not cleared
      let code = vec![
        0x3c, // INC A
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.registers.af = 0x0010; // carry flag set
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x0110);
    }
  }

  #[test]
  fn decrement_8_bit() {
    let code = vec![
      0x3e, 0x10, // LD A, 0x10
      0x01, 0x05, 0x05, // LD BC, 0x0505
      0x11, 0x06, 0x04, // LD DE, 0x0406
      0x21, 0x07, 0x07, // LD HL, 0x0707
      0x3d, // DEC A
      0x05, // DEC B
      0x05, // DEC B
      0x0d, // DEC C
      0x0d, // DEC C
      0x0d, // DEC C
      0x15, // DEC D
      0x15, // DEC D
      0x15, // DEC D
      0x15, // DEC D
      0x1d, // DEC E
      0x1d, // DEC E
      0x1d, // DEC E
      0x1d, // DEC E
      0x1d, // DEC E
      0x25, // DEC H
      0x25, // DEC H
      0x25, // DEC H
      0x25, // DEC H
      0x25, // DEC H
      0x25, // DEC H
      0x2d, // DEC L
      0x2d, // DEC L
      0x2d, // DEC L
      0x2d, // DEC L
      0x2d, // DEC L
      0x2d, // DEC L
      0x2d, // DEC L
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af() & 0xff00, 0x0f00);
    assert_eq!(core.registers.get_bc(), 0x0302);
    assert_eq!(core.registers.get_de(), 0x0001);
    assert_eq!(core.registers.get_hl(), 0x0100);
    assert_eq!(core.registers.get_ip(), 39);
  }

  #[test]
  fn decrement_8_bit_flags() {
    { // test negative flag set
      let code = vec![
        0x3e, 0x02, // LD A, 0x02
        0x3d, // DEC A
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x0140);
    }
    { // test half-carry set
      let code = vec![
        0x3e, 0x10, // LD A, 0x10
        0x3d, // DEC A
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x0f60);
    }
    { // test zero flag set
      let code = vec![
        0x3e, 0x01, // LD A, 0xff
        0x3d, // DEC A
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x00c0);
    }
    { // test carry not cleared
      let code = vec![
        0x3e, 0x02, // LD A, 0x02
        0x3d, // DEC A
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.registers.af = 0x0010; // carry flag set
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x0150);
    }
  }

  #[test]
  fn add_hl() {
    {
      let code = vec![
        0x21, 0x00, 0x28, // LD HL, 0x2800
        0x01, 0x55, 0x08, // LD BC, 0x0855
        0x09, // ADD HL, BC
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x0020);
      assert_eq!(core.registers.get_hl(), 0x2800 + 0x855);
    }
    {
      let code = vec![
        0x21, 0x80, 0x40, // LD HL, 0x4080
        0x11, 0xc1, 0x00, // LD DE, 0x00c1
        0x19, // ADD HL, DE
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.registers.af = 0x0010;
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x0000);
      assert_eq!(core.registers.get_hl(), 0x4080 + 0x00c1);
    }
    {
      let code = vec![
        0x21, 0x00, 0x80, // LD HL, 0x8000
        0x29, // ADD HL, HL
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x0010);
      assert_eq!(core.registers.get_hl(), 0);
    }
    {
      let code = vec![
        0x21, 0x80, 0x28, // LD HL, 0x2880
        0x31, 0x95, 0x00, // LD SP, 0x0095
        0x39, // ADD HL, SP
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x0000);
      assert_eq!(core.registers.get_hl(), 0x2880 + 0x95);
    }
    {
      let code = vec![
        0x21, 0x04, 0x18, // LD HL, 0x1804
        0x31, 0x50, 0x38, // LD SP, 0x3850
        0x39, // ADD HL, SP
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x0020);
      assert_eq!(core.registers.get_hl(), 0x1804 + 0x3850);
    }
  }

  #[test]
  fn add_a() {
    let code = vec![
      0x3e, 0x10, // LD A, 0x10
      0x06, 0x06, // LD B, 0x06
      0x80, // ADD A, B
      0xc3, 0x08, 0x00, // JMP 0x0008
      0x0e, 0x04, // LD C, 0x04
      0x81, // ADD A, C
      0xc3, 0x0e, 0x00, // JMP 0x000e
      0x16, 0x58, // LD D, 0x58
      0x82, // ADD A, D
      0xc3, 0x14, 0x00, // JMP 0x0014
      0x1e, 0x35, // LD E, 0x35
      0x83, // ADD A, E
      0xc3, 0x1a, 0x00, // JMP 0x001a
      0x26, 0x81, // LD H, 0x81
      0x84, // ADD A, H
      0xc3, 0x20, 0x00, // JMP 0x0020
      0x2e, 0x40, // LD L, 0x40
      0x85, // ADD A, L
      0xc3, 0x26, 0x00, // JMP 0x0026
      0x87, // ADD A, A
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x1600);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x1a00);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x7220);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xa700);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x2810);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x6800);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xd020);
  }

  #[test]
  fn adc_a() {
    let code = vec![
      0x3e, 0x60, // LD A, 0x60
      0x06, 0x0f, // LD B, 0x0f
      0x88, // ADC A, B
      0xc3, 0x08, 0x00, // JMP 0x0008
      0x0e, 0x0e, // LD C, 0x0e
      0x89, // ADC A, C
      0xc3, 0x0e, 0x00, // JMP 0x000e
      0x16, 0x04, // LD D, 0x04
      0x8a, // ADC A, D
      0xc3, 0x14, 0x00, // JMP 0x0014
      0x1e, 0x35, // LD E, 0x35
      0x8b, // ADC A, E
      0xc3, 0x1a, 0x00, // JMP 0x001a
      0x26, 0x80, // LD H, 0x80
      0x8c, // ADC A, H
      0xc3, 0x20, 0x00, // JMP 0x0020
      0x2e, 0x04, // LD L, 0x04
      0x8d, // ADC A, L
      0xc3, 0x26, 0x00, // JMP 0x0026
      0x87, // ADD A, A
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.registers.af = 0x0010;
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x7020);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x7e00);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x8220);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xb700);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x3710);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x3c00);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x7820);
  }

  #[test]
  fn and_a() {
    let code = vec![
      0x3e, 0xff, // LD A, 0xff
      0x06, 0xef, // LD B, 0xef
      0xa0, // AND A, B
      0xc3, 0x08, 0x00, // JMP 0x0008
      0x0e, 0xfc, // LD C, 0xfc
      0xa1, // AND A, C
      0xc3, 0x0e, 0x00, // JMP 0x000e
      0x16, 0x8f, // LD D, 0x8f
      0xa2, // AND A, D
      0xc3, 0x14, 0x00, // JMP 0x0014
      0x1e, 0x5d, // LD E, 0x5d
      0xa3, // AND A, E
      0xc3, 0x1a, 0x00, // JMP 0x001a
      0x26, 0x04, // LD H, 0x04
      0xa4, // AND A, H
      0xc3, 0x20, 0x00, // JMP 0x0020
      0x2e, 0xf0, // LD L, 0xf0
      0xa5, // AND A, L
      0xc3, 0x26, 0x00, // JMP 0x0026
      0xa7, // ADD A, A
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xef20);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xec20);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x8c20);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0c20);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0420);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x00a0);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x00a0);
  }

  #[test]
  fn xor_a() {
    let code = vec![
      0x3e, 0x0f, // LD A, 0x0f
      0x06, 0x11, // LD B, 0x11
      0xa8, // XOR A, B
      0xc3, 0x08, 0x00, // JMP 0x0008
      0x0e, 0x50, // LD C, 0x50
      0xa9, // XOR A, C
      0xc3, 0x0e, 0x00, // JMP 0x000e
      0x16, 0x80, // LD D, 0x80
      0xaa, // XOR A, D
      0xc3, 0x14, 0x00, // JMP 0x0014
      0x1e, 0x03, // LD E, 0x03
      0xab, // XOR A, E
      0xc3, 0x1a, 0x00, // JMP 0x001a
      0x26, 0x04, // LD H, 0x04
      0xac, // XOR A, H
      0xc3, 0x20, 0x00, // JMP 0x0020
      0x2e, 0xf0, // LD L, 0xf0
      0xad, // XOR A, L
      0xc3, 0x26, 0x00, // JMP 0x0026
      0xaf, // XOR A, A
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x1e00);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x4e00);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xce00);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xcd00);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xc900);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x3900);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0080);
  }

  #[test]
  fn or_a() {
    let code = vec![
      0x3e, 0x00, // LD A, 0x00
      0x06, 0x00, // LD B, 0x00
      0xb0, // OR A, B
      0xc3, 0x08, 0x00, // JMP 0x0008
      0x0e, 0x54, // LD C, 0x54
      0xb1, // OR A, C
      0xc3, 0x0e, 0x00, // JMP 0x000e
      0x16, 0x80, // LD D, 0x80
      0xb2, // OR A, D
      0xc3, 0x14, 0x00, // JMP 0x0014
      0x1e, 0x03, // LD E, 0x03
      0xb3, // OR A, E
      0xc3, 0x1a, 0x00, // JMP 0x001a
      0x26, 0x04, // LD H, 0x04
      0xb4, // OR A, H
      0xc3, 0x20, 0x00, // JMP 0x0020
      0x2e, 0xf0, // LD L, 0xf0
      0xb5, // OR A, L
      0xc3, 0x26, 0x00, // JMP 0x0026
      0xb7, // OR A, A
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0080);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x5400);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xd400);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xd700);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xd700);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xf700);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xf700);
  }

  #[test]
  fn rla() {
    let code = vec![
      0x3e, 0x0e, // MOV A, 0x0e
      0x17, // RLA
      0xc3, 0x06, 0x00, // JP 0x0006
      0x17, // RLA
      0xc3, 0x0a, 0x00, // JP 0x000a
      0x17, // RLA
      0xc3, 0x0e, 0x00, // JP 0x000e
      0x17, // RLA
      0xc3, 0x12, 0x00, // JP 0x0012
      0x17, // RLA
      0xc3, 0x16, 0x00, // JP 0x0016
      0x17, // RLA
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.registers.af = 0x00c0;
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x1c00);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x3800);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x7000);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xe000);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xc010);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x8110);
  }

  #[test]
  fn rlca() {
    let code = vec![
      0x3e, 0x0e, // MOV A, 0x0e
      0x07, // RLCA
      0xc3, 0x06, 0x00, // JP 0x0006
      0x07, // RLCA
      0xc3, 0x0a, 0x00, // JP 0x000a
      0x07, // RLCA
      0xc3, 0x0e, 0x00, // JP 0x000e
      0x07, // RLCA
      0xc3, 0x12, 0x00, // JP 0x0012
      0x07, // RLCA
      0xc3, 0x16, 0x00, // JP 0x0016
      0x07, // RLCA
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.registers.af = 0x00c0;
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x1c00);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x3800);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x7000);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xe000);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xc110);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x8310);
  }

  #[test]
  fn rlc() {
    let code = vec![
      0x06, 0x01, // LD B, 0x01
      0xcb, 0x00, // RLC B
      0xc3, 0x07, 0x00, // JMP 0x0007
      0x0e, 0xc3, // LD C, 0xc3
      0xcb, 0x01, // RLC C
      0xc3, 0x0e, 0x00, // JMP 0x000e
      0x16, 0x70, // LD D, 0x70
      0xcb, 0x02, // RLC D
      0xc3, 0x15, 0x00, // JMP 0x0015
      0x1e, 0xff, // LD E, 0xff
      0xcb, 0x03, // RLC E
      0xc3, 0x1c, 0x00, // JMP 0x001c
      0x26, 0x03, // LD H, 0x03
      0xcb, 0x04, // RLC H
      0xc3, 0x23, 0x00, // JMP 0x0023
      0x2e, 0xc0, // LD L, 0xc0
      0xcb, 0x05, // RLC L
      0xc3, 0x2a, 0x00, // JMP 0x002a
      0x3e, 0x00, // LD A, 0x00
      0xcb, 0x07, // RLC A
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_bc(), 0x0200);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0010);
    assert_eq!(core.registers.get_bc(), 0x0287);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0000);
    assert_eq!(core.registers.get_de(), 0xe000);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0010);
    assert_eq!(core.registers.get_de(), 0xe0ff);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0000);
    assert_eq!(core.registers.get_hl(), 0x0600);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0010);
    assert_eq!(core.registers.get_hl(), 0x0681);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0080);
  }

  #[test]
  fn rra() {
    let code = vec![
      0x3e, 0x0e, // MOV A, 0x0e
      0x1f, // RRA
      0xc3, 0x06, 0x00, // JP 0x0006
      0x1f, // RRA
      0xc3, 0x0a, 0x00, // JP 0x000a
      0x1f, // RRA
      0xc3, 0x0e, 0x00, // JP 0x000e
      0x1f, // RRA
      0xc3, 0x12, 0x00, // JP 0x0012
      0x1f, // RRA
      0xc3, 0x16, 0x00, // JP 0x0016
      0x1f, // RRA
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.registers.af = 0x00c0;
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0700);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0310);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x8110);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xc010);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xe000);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x7000);
  }

  #[test]
  fn rrca() {
    let code = vec![
      0x3e, 0x0a, // MOV A, 0x0a
      0x0f, // RRCA
      0xc3, 0x06, 0x00, // JP 0x0006
      0x0f, // RRCA
      0xc3, 0x0a, 0x00, // JP 0x000a
      0x0f, // RRCA
      0xc3, 0x0e, 0x00, // JP 0x000e
      0x0f, // RRCA
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.registers.af = 0x00a0;
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0500);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x8210);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x4100);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xa010);
  }

  #[test]
  fn bit_test() {
    let code = vec![
      0x06, 0x0f, // LD B, 0x0f
      0xcb, 0x40, // BIT 0,B
      0xc3, 0x07, 0x00, // JMP 0x0007
      0x0e, 0xf8, // LD C, 0xf8
      0xcb, 0x49, // BIT 1, C
      0xc3, 0x0e, 0x00, // JMP 0x000e
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0020);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x00a0);
  }

  #[test]
  fn bit_set() {
    let code = vec![
      0xcb, 0xc0, // SET 0, B
      0xc3, 0x05, 0x00, // JMP 0x0005
      0x0e, 0x44, // LD C, 0x44
      0xcb, 0xc9, // BIT 1, C
      0xc3, 0x0c, 0x00, // JMP 0x000c
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_bc(), 0x0100);
    core.run_code_block();
    assert_eq!(core.registers.get_bc(), 0x0146);
  }

  #[test]
  fn bit_clear() {
    let code = vec![
      0x06, 0x05, // LD B, 0x05
      0xcb, 0x80, // RES 0, B
      0xc3, 0x07, 0x00, // JMP 0x0007
      0x0e, 0x08, // LD C, 0x08
      0xcb, 0x89, // RES 1, C
      0xc3, 0x0e, 0x00, // JMP 0x000e
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_bc(), 0x0400);
    core.run_code_block();
    assert_eq!(core.registers.get_bc(), 0x0408);
  }

  #[test]
  fn complement_a() {
    let code = vec![
      0x3e, 0x14, // LD A, 0x14
      0x2f, // CPL
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.registers.af = 0x0080;
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xebe0);
  }

  #[test]
  fn set_carry_complement_carry() {
    let code = vec![
      0x37, // SCF
      0xc3, 0x04, 0x00,
      0x3f, // CCF
      0xc3, 0x08, 0x00,
      0x3f, // CCF
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0010);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0000);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0010);
  }

  #[test]
  fn load_to_indirect() {
    let code = vec![
      0x3e, 0x45, // LD A, 0x45
      0x06, 0xc0, // LD B, 0xc0
      0x0e, 0x05, // LD C, 0x05,
      0x02, // LD (BC), A
      0xc3, 0x0a, 0x00, // JP 0x000a
      0x16, 0xc1, // LD D, 0xc1,
      0x1e, 0x01, // LD E, 0x01,
      0x12, // LD (DE), A
      0xc3, 0x12, 0x00, // JP 0x0012
      0x26, 0xc2, // LD H, 0xc2
      0x2e, 0x10, // LD L, 0x10
      0x22, // LD (HL+), A
      0xc3, 0x1a, 0x00, // JP 0x001a
      0x32, // LD (HL-), A
      0xc3, 0x1e, 0x00, // JP 0x001e
      0x36, 0x2a, // LD (HL), 0x2a
      0xc3, 0x23, 0x00, // JP 0x0023
      0x70, // LD (HL), B
      0xc3, 0x27, 0x00, // JP 0x0027
      0x71, // LD (HL), C
      0xc3, 0x2b, 0x00, // JP 0x002b
      0x72, // LD (HL), D
      0xc3, 0x2f, 0x00, // JP 0x002f
      0x73, // LD (HL), E
      0xc3, 0x33, 0x00, // JP 0x0033
      0x74, // LD (HL), H
      0xc3, 0x37, 0x00, // JP 0x0037
      0x75, // LD (HL), L
      0xc3, 0x3b, 0x00, // JP 0x003b
      0x77, // LD (HL), A
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x05], 0x45);
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x101], 0x45);
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x210], 0x45);
    assert_eq!(core.registers.get_hl(), 0xc211);
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x211], 0x45);
    assert_eq!(core.registers.get_hl(), 0xc210);
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x210], 0x2a);
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x210], 0xc0);
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x210], 0x05);
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x210], 0xc1);
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x210], 0x01);
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x210], 0xc2);
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x210], 0x10);
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x210], 0x45);
  }

  #[test]
  fn absolute_jump() {
    let code = vec![
      0x3e, 0x0a, // MOV A, 0x0a
      0xc3, 0x07, 0x00, // JP 0x0007
      0x3e, 0x0b, // MOV A, 0x0b
      0x06, 0x10, // MOV B, 0x10
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0a00);
    assert_eq!(core.registers.get_bc(), 0x0000);
    assert_eq!(core.registers.get_ip(), 0x0007);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0a00);
    assert_eq!(core.registers.get_bc(), 0x1000);
    assert_eq!(core.registers.get_ip(), 0x0009);
  }

  #[test]
  fn conditional_jumps() {
    let code = vec![
      0x3e, 0x0a, // MOV A, 0x0a
      0xca, 0x07, 0x00, // JP Z, 0x0007
      0x3e, 0x0b, // MOV A, 0x0b
      0x06, 0x10, // MOV B, 0x10

      0xc2, 0x0d, 0x00, // JP NZ, 0x000d
      0x04, // INC B
      0x3e, 0x20, // MOV A, 0x20
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    // jump should fail
    assert_eq!(core.registers.get_ip(), 0x0005);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0b00);
    assert_eq!(core.registers.get_bc(), 0x1000);
    // jump should succeed
    assert_eq!(core.registers.get_ip(), 0x000d);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x2000);
    assert_eq!(core.registers.get_bc(), 0x1000);
  }
}

use crate::cache::CodeCache;
use crate::cart::Header;
use crate::cpu::{self, Registers};
use crate::mem::{MemoryAreas, memory_write_word};
use std::fs::File;

#[derive(Debug, Eq, PartialEq)]
pub enum RunState {
  Run,
  Stop,
  Halt,
}

pub struct Core {
  pub cache: CodeCache,
  pub registers: Registers,
  pub memory: MemoryAreas,
  pub interrupts_enabled: bool,
  pub run_state: RunState,
}

impl Core {
  pub fn with_code_block(code: Box<[u8]>) -> Self {
    Self {
      cache: CodeCache::new(),
      registers: Registers::new(),
      memory: MemoryAreas::with_rom(code),
      interrupts_enabled: false,
      run_state: RunState::Run,
    }
  }

  pub fn from_rom_file(rom_file: &mut File, header: Header) -> Self {
    Self {
      cache: CodeCache::new(),
      registers: Registers::after_boot(),
      memory: MemoryAreas::with_rom_file(rom_file, &header),
      interrupts_enabled: false,
      run_state: RunState::Run,
    }
  }

  /// If interrupts are enabled, check the current interrupt flags and enter the
  /// highest-priority active interrupt.
  pub fn handle_interrupt(&mut self) {
    if !self.interrupts_enabled {
      return;
    }
    let interrupts = self.memory.io.get_active_interrupts();
    if interrupts == 0 {
      return;
    }
    let vector: u32 = if interrupts & 1 != 0 {
      0x40 // VBLANK
    } else if interrupts & 2 != 0 {
      0x48 // LCD STAT
    } else if interrupts & 4 != 0 {
      0x50 // timer
    } else if interrupts & 8 != 0 {
      0x58 // serial transfer
    } else {
      0x60 // input
    };
    // for timing accuracy, skip five machine cycles

    self.push_ip();
    self.registers.ip = vector;
  }

  /// Push the instruction pointer onto the stack, such as at the start of an
  /// interrupt request.
  pub fn push_ip(&mut self) {
    self.registers.sp = self.registers.sp.wrapping_sub(2);
    let ip = self.registers.ip as u16;
    let sp = self.registers.sp as u16;
    let mem_ptr = &mut self.memory as *mut MemoryAreas;
    memory_write_word(mem_ptr, sp, ip);
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
    let result = self.cache.call(address, &mut self.registers);
    match result {
      cpu::STATUS_STOP => {
        self.run_state = RunState::Stop;
        println!("STOP");
      },
      cpu::STATUS_HALT => {
        self.run_state = RunState::Halt;
        println!("HALT");
      },
      cpu::STATUS_INTERRUPT_DISABLE => {
        self.interrupts_enabled = false;
        println!("DISABLE INT");
      },
      cpu::STATUS_INTERRUPT_ENABLE => {
        self.interrupts_enabled = true;
        println!("ENABLE INT");
      },
      _ => (),
    }
    self.handle_interrupt();
    //println!("[{}] {:?}", result, self.registers);
  }
}

#[cfg(test)]
mod tests {
  use super::{Core, RunState};

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
  fn increment_hl_indirect() {
    let code = vec![
      0x3e, 0x4f, // LD A, 0x4f
      0xea, 0x10, 0xc0, // LD (0xc010), A
      0x26, 0xc0, // LD H, 0xc0
      0x2e, 0x10, // LD L, 0x10
      0x34, // INC (HL)
      0xc3, 0x0d, 0x00, // JP 0x000d
      0x3e, 0xff, // LD A, 0xff
      0xea, 0x10, 0xc0, // LD (0xc010), A
      0x34, // INC (HL)
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.registers.af = 0x0040;
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x10], 0x50);
    assert_eq!(core.registers.get_af(), 0x4f20);
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x10], 0);
    assert_eq!(core.registers.get_af(), 0xffa0);
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
  fn add_indirect() {
    let code = vec![
      0x3e, 0x0f, // LD A, 0x0f
      0x1e, 0xfc, // LD E, 0xfc
      0xea, 0x40, 0xc1, // LD (0xc140), A
      0x26, 0xc1, // LD H, 0xc1
      0x2e, 0x40, // LD L, 0x40
      0x3e, 0x01, // LD A, 0x01
      0x86, // ADD A, (HL)
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x1020);
    assert_eq!(core.registers.get_de(), 0x00fc);
  }

  #[test]
  fn add_8() {
    let code = vec![
      0xc6, 0x0f, // ADD A, 0x0f
      0xc3, 0x05, 0x00, // JP 0x0005
      0xc6, 0x23, // ADD A, 0x23
      0xc3, 0x0a, 0x00, // JP 0x000a
      0xc6, 0xf0, // ADD A, 0xf0
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0f00);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x3220);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x2210);
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
  fn adc_indirect() {
    let code = vec![
      0x3e, 0x0d, // LD A, 0x0d
      0x1e, 0xfc, // LD E, 0xfc
      0xea, 0x00, 0xc2, // LD (0xc200), A
      0x26, 0xc2, // LD H, 0xc2
      0x2e, 0x00, // LD L, 0x00
      0x3e, 0x02, // LD A, 0x02
      0x8e, // ADC A, (HL)
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.registers.af = 0x0010;
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x1020);
    assert_eq!(core.registers.get_de(), 0x00fc);
  }

  #[test]
  fn adc_8() {
    let code = vec![
      0xce, 0x14, // ADC A, 0x14
      0xc3, 0x05, 0x00, // JP 0x0005
      0xce, 0xf1, // ADC A, 0xf1
      0xc3, 0x0a, 0x00, // JP 0x000a
      0xce, 0x30, // ADC A, 0x30
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x1400);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0510);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x3600);
  }

  #[test]
  fn sub_a() {
    let code = vec![
      0x3e, 0x48, // LD A, 0x48
      0x06, 0x07, // LD B, 0x07
      0x90, // SUB B
      0xc3, 0x08, 0x00, // JP 0x0008
      0x0e, 0x04, // LD C, 0x04
      0x91, // SUB C
      0xc3, 0x0e, 0x00, // JP 0x000e
      0x16, 0x3d, // LD D, 0x3d
      0x92, // SUB D
      0xc3, 0x14, 0x00, // JP 0x0014
      0x1e, 0x05, // LD E, 0x05
      0x93, // SUB E
      0xc3, 0x1a, 0x00, // JP 0x001a
      0x26, 0x4c, // LD H, 0x4c
      0x94, // SUB H
      0xc3, 0x20, 0x00, // JP 0x0020
      0x2e, 0x10, // LD L, 0x10
      0x95, // SUB L
      0xc3, 0x26, 0x00, // JP 0x0026
      0x97, // SUB A
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x4140);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x3d60);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x00c0);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xfb70);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xaf60);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x9f40);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x00c0);
  }

  #[test]
  fn sub_indirect() {
    let code = vec![
      0x3e, 0x12, // LD A, 0x12
      0x1e, 0xfc, // LD E, 0xfc
      0xea, 0x3e, 0xc0, // LD (0xc03e), A
      0x26, 0xc0, // LD H, 0xc0
      0x2e, 0x3e, // LD L, 0x3e
      0x3e, 0x38, // LD A, 0x38
      0x96, // SUB A, (HL)
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x2640);
    assert_eq!(core.registers.get_de(), 0x00fc);
  }

  #[test]
  fn sub_8() {
    let code = vec![
      0x3e, 0x4e, // LD A, 0x4e
      0xd6, 0x1f, // SUB 0x1f
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x2f60);
  }

  #[test]
  fn sbc_a() {
    let code = vec![
      0x3e, 0x05, // LD A, 0x05
      0x06, 0x07, // LD B, 0x07
      0x98, // SBC B
      0xc3, 0x08, 0x00, // JP 0x0008
      0x0e, 0x04, // LD C, 0x04
      0x99, // SBC C
      0xc3, 0x0e, 0x00, // JP 0x000e
      0x16, 0x59, // LD D, 0x59
      0x9a, // SBC D
      0xc3, 0x14, 0x00, // JP 0x0014
      0x1e, 0x21, // LD E, 0x21
      0x9b, // SBC E
      0xc3, 0x1a, 0x00, // JP 0x001a
      0x26, 0x70, // LD H, 0x70
      0x9c, // SBC H
      0xc3, 0x20, 0x00, // JP 0x0020
      0x2e, 0x10, // LD L, 0x10
      0x9d, // SBC L
      0xc3, 0x26, 0x00, // JP 0x0026
      0x9f, // SBC A
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xfe70);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xf940);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xa040);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x7f60);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0f40);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xff50);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xff70);
  }

  #[test]
  fn sbc_indirect() {
    let code = vec![
      0x3e, 0x14, // LD A, 0x14
      0x1e, 0xfc, // LD E, 0xfc
      0xea, 0x02, 0xc0, // LD (0xc002), A
      0x26, 0xc0, // LD H, 0xc0
      0x2e, 0x02, // LD L, 0x3e
      0x3e, 0x1f, // LD A, 0x1f
      0x9e, // SBC A, (HL)
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.registers.af = 0x0010;
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0a40);
    assert_eq!(core.registers.get_de(), 0x00fc);
  }

  #[test]
  fn sub_immediate() {
    let code = vec![
      0x3e, 0xf1, // LD A, 0xf1
      0xd6, 0x14, // SUB 0x14
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xdd60);
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
  fn and_indirect() {
    let code = vec![
      0x3e, 0xc3, // LD A, 0xc3
      0x1e, 0xfc, // LD E, 0xfc
      0xea, 0x08, 0xc0, // LD (0xc008), A
      0x26, 0xc0, // LD H, 0xc0
      0x2e, 0x08, // LD L, 0x08
      0x3e, 0x88, // LD A, 0x88
      0xa6, // AND A, (HL)
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x8020);
    assert_eq!(core.registers.get_de(), 0x00fc);
  }

  #[test]
  fn and_8() {
    let code = vec![
      0x3e, 0x7c, // LD A, 0x7c
      0xe6, 0x35, // AND 0x35
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x3420);
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
  fn xor_indirect() {
    let code = vec![
      0x3e, 0xc3, // LD A, 0xc3
      0x1e, 0xfc, // LD E, 0xfc
      0xea, 0x20, 0xc0, // LD (0xc020), A
      0x26, 0xc0, // LD H, 0xc0
      0x2e, 0x20, // LD L, 0x20
      0x3e, 0x88, // LD A, 0x88
      0xae, // XOR A, (HL)
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x4b00);
    assert_eq!(core.registers.get_de(), 0x00fc);
  }

  #[test]
  fn xor_8() {
    let code = vec![
      0x3e, 0xf0, // LD A, 0xf0
      0xee, 0xf0, // XOR 0xf0
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
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
  fn or_indirect() {
    let code = vec![
      0x3e, 0xc3, // LD A, 0xc3
      0x1e, 0xfc, // LD E, 0xfc
      0xea, 0x20, 0xc0, // LD (0xc020), A
      0x26, 0xc0, // LD H, 0xc0
      0x2e, 0x20, // LD L, 0x20
      0x3e, 0x88, // LD A, 0x88
      0xb6, // OR A, (HL)
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xcb00);
    assert_eq!(core.registers.get_de(), 0x00fc);
  }

  #[test]
  fn or_8() {
    let code = vec![
      0x3e, 0x73, // LD A, 0x73
      0xf6, 0xf1, // OR 0xf1
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0xf300);
  }

  #[test]
  fn cmp() {
    let code = vec![
      0x3e, 0x16, // LD A, 0x16
      0x06, 0x04, // LD B, 0x04
      0xb8, // CP B
      0xc3, 0x08, 0x00, // JP 0x0008
      0x0e, 0x08, // LD C, 0x08
      0xb9, // CP C
      0xc3, 0x0e, 0x00, // JP 0x000e
      0x16, 0x16, // LD D, 0x16
      0xba, // CP D
      0xc3, 0x14, 0x00, // JP 0x0014
      0x1e, 0x20, // LD E, 0x20
      0xbb, // CP E
      0xc3, 0x1a, 0x00, // JP 0x001a
      0x26, 0x28, // LD H, 0x28,
      0xbc, // CP H
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x1640);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x1660);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x16c0);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x1650);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x1670);
  }

  #[test]
  fn cmp_indirect() {
    let code = vec![
      0x3e, 0x50, // LD A, 0x50
      0x1e, 0xfc, // LD E, 0xfc
      0xea, 0x00, 0xc0, // LD (0xc000), A
      0x26, 0xc0, // LD H, 0xc0
      0x2e, 0x00, // LD L, 0x00
      0x3e, 0x34, // LD A, 0x34
      0xbe, // CP A, (HL)
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x3450);
    assert_eq!(core.registers.get_de(), 0x00fc);
  }

  #[test]
  fn cmp_8() {
    let code = vec![
      0x3e, 0x44, // LD A, 0x44
      0xfe, 0x44, // CP A, 0x44
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x44c0);
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
  fn sla() {
    let code = vec![
      0x06, 0x73, // LD B, 0x73,
      0xcb, 0x20, // SLA B
      0x18, 0x00, // JR 0
      0x0e, 0xf0, // LD C, 0xf0
      0xcb, 0x21, // SLA C
      0x18, 0x00, // JR 0
      0x16, 0x80, // LD D, 0x80
      0xcb, 0x22, // SLA D
      0x18, 0x00, // JR 0
      0x1e, 0x00, // LD E, 0x00
      0xcb, 0x23, // SLA E
      0x18, 0x00, // JR 0
      0x26, 0x33, // LD H, 0xee
      0xcb, 0x24, // SLA H
      0x18, 0x00, // JR 0
      0x2e, 0x99, // LD L, 0x99
      0xcb, 0x25, // SLA L
      0x18, 0x00, // JR 0
      0x3e, 0x1f, // LD A, 0x1f
      0xcb, 0x27, // SLA A
      0x18, 0x00, // JR 0
      0x26, 0xc0, // LD H, 0xc0
      0x2e, 0x20, // LD L, 0x20
      0x36, 0xc4, // LD (HL), 0xc4
      0xcb, 0x26, // SLA (HL)
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.registers.af = 0x00f0;
    core.run_code_block();
    assert_eq!(core.registers.get_bc(), 0xe600);
    assert_eq!(core.registers.get_af(), 0x0000);
    core.run_code_block();
    assert_eq!(core.registers.get_bc(), 0xe6e0);
    assert_eq!(core.registers.get_af(), 0x0010);
    core.run_code_block();
    assert_eq!(core.registers.get_de(), 0x0000);
    assert_eq!(core.registers.get_af(), 0x0090);
    core.run_code_block();
    assert_eq!(core.registers.get_de(), 0x0000);
    assert_eq!(core.registers.get_af(), 0x0080);
    core.run_code_block();
    assert_eq!(core.registers.get_hl(), 0x6600);
    assert_eq!(core.registers.get_af(), 0x0000);
    core.run_code_block();
    assert_eq!(core.registers.get_hl(), 0x6632);
    assert_eq!(core.registers.get_af(), 0x0010);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x3e00);
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x20], 0x88);
    assert_eq!(core.registers.get_af(), 0x3e10);
  }

  #[test]
  fn sra() {
    let code = vec![
      0x06, 0x73, // LD B, 0x73,
      0xcb, 0x28, // SRA B
      0x18, 0x00, // JR 0
      0x0e, 0xf0, // LD C, 0xf0
      0xcb, 0x29, // SRA C
      0x18, 0x00, // JR 0
      0x16, 0x01, // LD D, 0x01
      0xcb, 0x2a, // SRA D
      0x18, 0x00, // JR 0
      0x1e, 0x00, // LD E, 0x00
      0xcb, 0x2b, // SRA E
      0x18, 0x00, // JR 0
      0x26, 0xee, // LD H, 0xee
      0xcb, 0x2c, // SRA H
      0x18, 0x00, // JR 0
      0x2e, 0x99, // LD L, 0x99
      0xcb, 0x2d, // SRA L
      0x18, 0x00, // JR 0
      0x3e, 0x1e, // LD A, 0x1e
      0xcb, 0x2f, // SRA A
      0x18, 0x00, // JR 0
      0x26, 0xc0, // LD H, 0xc0
      0x2e, 0x20, // LD L, 0x20
      0x36, 0xc4, // LD (HL), 0xc4
      0xcb, 0x2e, // SRA (HL)
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_bc(), 0x3900);
    assert_eq!(core.registers.get_af(), 0x0010);
    core.run_code_block();
    assert_eq!(core.registers.get_bc(), 0x39f8);
    assert_eq!(core.registers.get_af(), 0x0000);
    core.run_code_block();
    assert_eq!(core.registers.get_de(), 0x0000);
    assert_eq!(core.registers.get_af(), 0x0090);
    core.run_code_block();
    assert_eq!(core.registers.get_de(), 0x0000);
    assert_eq!(core.registers.get_af(), 0x0080);
    core.run_code_block();
    assert_eq!(core.registers.get_hl(), 0xf700);
    assert_eq!(core.registers.get_af(), 0x0000);
    core.run_code_block();
    assert_eq!(core.registers.get_hl(), 0xf7cc);
    assert_eq!(core.registers.get_af(), 0x0010);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0f00);
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x20], 0xe2);
    assert_eq!(core.registers.get_af(), 0x0f00);
  }

  #[test]
  fn srl() {
    let code = vec![
      0x06, 0x73, // LD B, 0x73,
      0xcb, 0x38, // SRA B
      0x18, 0x00, // JR 0
      0x0e, 0xf0, // LD C, 0xf0
      0xcb, 0x39, // SRA C
      0x18, 0x00, // JR 0
      0x16, 0x01, // LD D, 0x01
      0xcb, 0x3a, // SRA D
      0x18, 0x00, // JR 0
      0x1e, 0x00, // LD E, 0x00
      0xcb, 0x3b, // SRA E
      0x18, 0x00, // JR 0
      0x26, 0xee, // LD H, 0xee
      0xcb, 0x3c, // SRA H
      0x18, 0x00, // JR 0
      0x2e, 0x99, // LD L, 0x99
      0xcb, 0x3d, // SRA L
      0x18, 0x00, // JR 0
      0x3e, 0x1e, // LD A, 0x1e
      0xcb, 0x3f, // SRA A
      0x18, 0x00, // JR 0
      0x26, 0xc0, // LD H, 0xc0
      0x2e, 0x20, // LD L, 0x20
      0x36, 0xc4, // LD (HL), 0xc4
      0xcb, 0x3e, // SRA (HL)
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_bc(), 0x3900);
    assert_eq!(core.registers.get_af(), 0x0010);
    core.run_code_block();
    assert_eq!(core.registers.get_bc(), 0x3978);
    assert_eq!(core.registers.get_af(), 0x0000);
    core.run_code_block();
    assert_eq!(core.registers.get_de(), 0x0000);
    assert_eq!(core.registers.get_af(), 0x0090);
    core.run_code_block();
    assert_eq!(core.registers.get_de(), 0x0000);
    assert_eq!(core.registers.get_af(), 0x0080);
    core.run_code_block();
    assert_eq!(core.registers.get_hl(), 0x7700);
    assert_eq!(core.registers.get_af(), 0x0000);
    core.run_code_block();
    assert_eq!(core.registers.get_hl(), 0x774c);
    assert_eq!(core.registers.get_af(), 0x0010);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x0f00);
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x20], 0x62);
    assert_eq!(core.registers.get_af(), 0x0f00);
  }

  #[test]
  fn swap() {
    let code = vec![
      0x06, 0xd7, // LD B, 0xd7
      0xcb, 0x30, // SWAP B
      0xc3, 0x07, 0x00, // JP 0x0007
      0x0e, 0x00, // LD C, 0x00
      0xcb, 0x31, // SWAP C
      0xc3, 0x0e, 0x00, // JP 0x000e
      0x16, 0xff, // LD D, 0xff
      0xcb, 0x32, // SWAP D
      0xc3, 0x15, 0x00, // JP 0x0015
      0x1e, 0x38, // LD E, 0x38
      0xcb, 0x33, // SWAP E
      0xc3, 0x1c, 0x00, // JP 0x001c
      0x26, 0x40, // LD H, 0x40
      0xcb, 0x34, // SWAP H
      0xc3, 0x23, 0x00, // JP 0x0023
      0x2e, 0x1f, // LD L, 0x1f
      0xcb, 0x35, // SWAP L
      0xc3, 0x2a, 0x00, // JP 0x002a
      0x3e, 0x36, // LD A, 0x36
      0xcb, 0x37, // SWAP A
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.registers.af = 0x00f0;
    core.run_code_block();
    assert_eq!(core.registers.get_bc(), 0x7d00);
    assert_eq!(core.registers.get_af(), 0x0000);
    core.run_code_block();
    assert_eq!(core.registers.get_bc(), 0x7d00);
    assert_eq!(core.registers.get_af(), 0x0080);
    core.run_code_block();
    assert_eq!(core.registers.get_de(), 0xff00);
    assert_eq!(core.registers.get_af(), 0x0000);
    core.run_code_block();
    assert_eq!(core.registers.get_de(), 0xff83);
    assert_eq!(core.registers.get_af(), 0x0000);
    core.run_code_block();
    assert_eq!(core.registers.get_hl(), 0x0400);
    assert_eq!(core.registers.get_af(), 0x0000);
    core.run_code_block();
    assert_eq!(core.registers.get_hl(), 0x04f1);
    assert_eq!(core.registers.get_af(), 0x0000);
    core.run_code_block();
    assert_eq!(core.registers.get_af(), 0x6300);
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
  fn daa() {
    { // test no correction case
      let code = vec![
        0xc6, 0x32, // ADD A, 0x32
        0xc6, 0x05, // ADD A, 0x05
        0x27, // DAA
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x3700);
    }
    { // test lower digit BCD overflow
      let code = vec![
        0xc6, 0x36, // ADD A, 0x36
        0xc6, 0x05, // ADD A, 0x05
        0x27, // DAA
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x4100);
    }
    { // test higher digit BCD overflow
      let code = vec![
        0xc6, 0x76, // ADD A, 0x76
        0xc6, 0x50, // ADD A, 0x50
        0x27, // DAA
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x2610);
    }
    { // lower digit correct causes carry
      let code = vec![
        0xc6, 0x86, // ADD A, 0x86
        0xc6, 0x76, // ADD A, 0x76
        0x27, // DAA
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x6210);
    }
    { // test both digits BCD overflow
      let code = vec![
        0xc6, 0x36, // ADD A, 0x36
        0xc6, 0x88, // ADD A, 0x88
        0x27, // DAA
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x2410);
    }
    { // test lower digit half-carry
      let code = vec![
        0xc6, 0x39, // ADD A, 0x39
        0xc6, 0x19, // ADD A, 0x19
        0x27, // DAA
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x5800);
    }
    { // test higher digit carry
      let code = vec![
        0xc6, 0x91, // ADD A, 0x91
        0xc6, 0x93, // ADD A, 0x93
        0x27, // DAA
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x8410);
    }
    { // overflow all digits
      let code = vec![
        0xc6, 0x99, // ADD A, 0x99
        0xc6, 0x99, // ADD A, 0x99
        0x27, // DAA
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x9810);
    }
    { // subtract, adjust lower digit
      let code = vec![
        0xc6, 0x10, // ADD A, 0x10
        0xd6, 0x02, // SUB 0x02
        0x27, // DAA
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x0840);
    }
    { // subtract, adjust higher digit
      let code = vec![
        0xc6, 0x10, // ADD A, 0x10
        0xd6, 0x20, // SUB 0x20
        0x27, // DAA
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x9050);
    }
    {// add, adjust to zero
      let code = vec![
        0xc6, 0x85, // ADD A, 0x85
        0xc6, 0x15, // ADD A, 0x15
        0x27, // DAA
      ];
      let mut core = Core::with_code_block(code.into_boxed_slice());
      core.run_code_block();
      assert_eq!(core.registers.get_af(), 0x0090);
    }
  }

  #[test]
  fn add_sp() {
    let code = vec![
      0x31, 0x50, 0xc0, // LD SP, 0xc050
      0xe8, 0x2c, // ADD SP, 0x2c
      0xc3, 0x08, 0x00, // JP 0x0008
      0xe8, 0xff, // ADD SP, 0xff
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc07c);
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc07b);
  }

  #[test]
  fn load_to_sp() {
    let code = vec![
      0x21, 0x43, 0x65, // LD HL, 0x6543
      0xf9, // LD SP, HL
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0x6543);
  }

  #[test]
  fn load_sp_offset() {
    let code = vec![
      0x31, 0x14, 0xc0, // LD SP, 0xc014
      0xf8, 0x23, // LD HL, SP+0x23
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_hl(), 0xc037);
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
  fn load_from_indirect() {
    let code = vec![
      0x06, 0xc0, // LD B, 0xc0
      0x0e, 0x03, // LD C, 0x03,
      0x0a, // LD A, (BC)
      0xc3, 0x08, 0x00, // JP 0x0008
      0x16, 0x00, // LD D, 0x00
      0x1e, 0x04, // LD E, 0x04
      0x1a, // LD A, (DE)
      0xc3, 0x10, 0x00, // JP 0x0010
      0x26, 0x00, // LD H, 0x00
      0x2e, 0x12, // LD L, 0x12
      0x2a, // LD A, (HL+)
      0xc3, 0x18, 0x00, // JP 0x0018
      0x3a, // LD A, (HL-)
      0xc3, 0x1c, 0x00, // JP 0x001c
      0x46, // LD B, (HL)
      0xc3, 0x20, 0x00, // JP 0x0020
      0x4e, // LD C, (HL)
      0xc3, 0x24, 0x00, // JP 0x0024
      0x56, // LD D, (HL)
      0xc3, 0x28, 0x00, // JP 0x0028
      0x5e, // LD E, (HL)
      0xc3, 0x2c, 0x00, // JP 0x002c
      0x7e, // LD A, (HL)
      0xc3, 0x30, 0x00, // JP 0x0030
      0x6e, // LD L, (HL)
      0xc3, 0x34, 0x00, // JP 0x0030
      0x66, // LD H, (HL)
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    for i in 0..10 {
      core.memory.work_ram[i] = i as u8;
    }
    core.run_code_block();
    assert_eq!(core.registers.get_af() & 0xff00, 0x0300);
    core.run_code_block();
    assert_eq!(core.registers.get_af() & 0xff00, 0x0a00);
    core.run_code_block();
    assert_eq!(core.registers.get_af() & 0xff00, 0x2e00);
    assert_eq!(core.registers.get_hl(), 0x0013);
    core.run_code_block();
    assert_eq!(core.registers.get_af() & 0xff00, 0x1200);
    assert_eq!(core.registers.get_hl(), 0x0012);
    core.run_code_block();
    assert_eq!(core.registers.get_bc(), 0x2e03);
    core.run_code_block();
    assert_eq!(core.registers.get_bc(), 0x2e2e);
    core.run_code_block();
    assert_eq!(core.registers.get_de(), 0x2e04);
    core.run_code_block();
    assert_eq!(core.registers.get_de(), 0x2e2e);
    core.run_code_block();
    assert_eq!(core.registers.get_af() & 0xff00, 0x2e00);
    core.run_code_block();
    assert_eq!(core.registers.get_hl(), 0x002e);
    core.run_code_block();
    assert_eq!(core.registers.get_hl(), 0x302e);
  }

  #[test]
  fn write_stack_pointer_to_memory() {
    let code = vec![
      0x31, 0x20, 0x44, // LD SP, 0x4420
      0x08, 0x40, 0xc0, // LD (0xc040), SP
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x40], 0x20);
    assert_eq!(core.memory.work_ram[0x41], 0x44);
  }

  #[test]
  fn store_and_retrieve_a_from_memory() {
    let code = vec![
      0x3e, 0x50, // LD A, 0x50
      0xea, 0x15, 0xc0, // LD (0xc015), A
      0xc3, 0x08, 0x00, // JP 0x0008
      0xfa, 0x02, 0x00, // LD A, (0x0002)
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.memory.work_ram[0x15], 0x50);
    core.run_code_block();
    assert_eq!(core.registers.get_af() & 0xff00, 0xea00);
  }

  #[test]
  fn push_pop() {
    let code = vec![
      0x31, 0x00, 0xc1, // LD SP, 0xc100
      0x06, 0x14, // LD B, 0x14
      0x0e, 0x53, // LD C, 0x53
      0xc5, // PUSH BC
      0xc3, 0x0b, 0x00, // JP 0x000b
      0x16, 0x66, // LD D, 0x66
      0x1e, 0x33, // LD E, 0x33
      0xd5, // PUSH DE
      0xc3, 0x13, 0x00, // JP 0x0013
      0x26, 0x40, // LD H, 0x40
      0x2e, 0xfa, // LD L, 0xfa
      0xe5, // PUSH HL
      0xc3, 0x1b, 0x00, // JP 0x001b
      0x3e, 0x50, // LD A, 0x50
      0xa7, // AND A
      0xf5, // PUSH AF
      0xc3, 0x22, 0x00, // JP 0x0022
      0xc1, // POP BC
      0xc3, 0x26, 0x00, // JP 0x0026
      0xd1, // POP DE
      0xc3, 0x2a, 0x00, // JP 0x002a
      0xe1, // POP HL
      0xc3, 0x2e, 0x00, // JP 0x002e,
      0xf1, // POP AF
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc0fe);
    assert_eq!(core.memory.work_ram[0xff], 0x14);
    assert_eq!(core.memory.work_ram[0xfe], 0x53);
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc0fc);
    assert_eq!(core.memory.work_ram[0xfd], 0x66);
    assert_eq!(core.memory.work_ram[0xfc], 0x33);
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc0fa);
    assert_eq!(core.memory.work_ram[0xfb], 0x40);
    assert_eq!(core.memory.work_ram[0xfa], 0xfa);
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc0f8);
    assert_eq!(core.memory.work_ram[0xf9], 0x50);
    assert_eq!(core.memory.work_ram[0xf8], 0x20);
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc0fa);
    assert_eq!(core.registers.get_bc(), 0x5020);
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc0fc);
    assert_eq!(core.registers.get_de(), 0x40fa);
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc0fe);
    assert_eq!(core.registers.get_hl(), 0x6633);
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc100);
    assert_eq!(core.registers.get_af(), 0x1450);
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

  #[test]
  fn relative_jump() {
    let code = vec![
      0x00,
      0x00,
      0x00,
      0x00,
      0x18, 0x04,
      0x00,
      0x00,
      0x00,
      0x00,
      0x18, 0xfe,
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x0a);
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x0a);
  }

  #[test]
  fn conditional_relative_jump() {
    let code = vec![
      0x28, 0x10, // JR Z, 0x10
      0xa7, // AND A
      0x28, 0x10, // JR Z, 0x10
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x02);
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x15);
  }

  #[test]
  fn jump_hl() {
    let code = vec![
      0x21, 0x40, 0x03, // LD HL, 0x0340
      0xe9, // JP (HL)
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x0340);
  }

  #[test]
  fn subroutine_call() {
    let code = vec![
      0x31, 0x00, 0xc1, // LD SP, 0xc100
      0xcd, 0x10, 0x00, // CALL 0x0010
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc0fe);
    assert_eq!(core.registers.get_ip(), 0x10);
    assert_eq!(core.memory.work_ram[0xff], 0x00);
    assert_eq!(core.memory.work_ram[0xfe], 0x06);
  }

  #[test]
  fn conditional_call() {
    let code = vec![
      0x31, 0x00, 0xc1, // LD SP, 0xc100
      0xcc, 0x14, 0x00, // CALL Z, 0x0014
      0xdc, 0x50, 0x00, // CALL C, 0x0050
      0xc4, 0x0f, 0x00, // CALL NZ, 0x000f
      0x00, 0x00, 0x00, 0x00,
      0xa7, // AND A
      0xd4, 0x03, 0x00, // CALL NC, 0x0003
      0xc6, 0xf0, // ADD A, 0xf0
      0xc6, 0xf0, // ADD A, 0xf0
      0xdc, 0x40, 0x00, // CALL C, 0x40
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc100);
    assert_eq!(core.registers.get_ip(), 0x06);
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc100);
    assert_eq!(core.registers.get_ip(), 0x09);
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc0fe);
    assert_eq!(core.registers.get_ip(), 0x0f);
    assert_eq!(core.memory.work_ram[0xff], 0x00);
    assert_eq!(core.memory.work_ram[0xfe], 0x0c);
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc0fc);
    assert_eq!(core.registers.get_ip(), 0x03);
    assert_eq!(core.memory.work_ram[0xfd], 0x00);
    assert_eq!(core.memory.work_ram[0xfc], 0x14);
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc0fa);
    assert_eq!(core.registers.get_ip(), 0x14);
    assert_eq!(core.memory.work_ram[0xfb], 0x00);
    assert_eq!(core.memory.work_ram[0xfa], 0x06);
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc0f8);
    assert_eq!(core.registers.get_ip(), 0x40);
    assert_eq!(core.memory.work_ram[0xf9], 0x00);
    assert_eq!(core.memory.work_ram[0xf8], 0x1b);
  }

  #[test]
  fn rst() {
    let code = vec![
      0x31, 0x00, 0xc1, // LD SP, 0xc100
      0xe7, // RST 0x20
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc0fe);
    assert_eq!(core.registers.get_ip(), 0x20);
    assert_eq!(core.memory.work_ram[0xff], 0x00);
    assert_eq!(core.memory.work_ram[0xfe], 0x04);
  }

  #[test]
  fn ret() {
    let code = vec![
      0x31, 0x00, 0xc1, // LD SP, 0xc100
      0xcd, 0x10, 0x00, // CALL 0x0010
      0xcd, 0x11, 0x00, // CALL 0x0011
      0xcd, 0x13, 0x00, // CALL 0x0013
      0xc3, 0x15, 0x00, // JP 0x0015
      0x00,
      0xc9, // RET
      0xc8, // RET Z
      0xc0, // RET NZ
      0xd8, // RET C
      0xd0, // RET NC
      0xa7, // AND A
      0xcd, 0x11, 0x00, // CALL 0x0011
      0xc6, 0xff, // ADD A, 0xff
      0xc6, 0x01, // ADD A, 0x01
      0xcd, 0x12, 0x00, // CALL 0x0012
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x10);
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x06);
    assert_eq!(core.registers.get_sp(), 0xc100);
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x11);
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x12);
    assert_eq!(core.registers.get_sp(), 0xc0fe);
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x09);
    assert_eq!(core.registers.get_sp(), 0xc100);
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x13);
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x14);
    assert_eq!(core.registers.get_sp(), 0xc0fe);
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x0c);
    assert_eq!(core.registers.get_sp(), 0xc100);
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x15);
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x11);
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x19);
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x12);
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x13);
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x20);
  }

  #[test]
  fn reti() {
    let code = vec![
      0x31, 0x00, 0xc1, // LD SP, 0xc100
      0xcd, 0x08, 0x00, // CALL 0x0008
      0x00, 0x00,
      0xd9, // RETI
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.interrupts_enabled = false;
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x08);
    core.run_code_block();
    assert_eq!(core.registers.get_sp(), 0xc100);
    assert_eq!(core.registers.get_ip(), 0x06);
    assert!(core.interrupts_enabled);
  }
  
  #[test]
  fn stop() {
    let code = vec![
      0x10, 0x00, // STOP
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.run_state, RunState::Stop);
  }

  #[test]
  fn halt() {
    let code = vec![
      0x76, // HALT
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block();
    assert_eq!(core.run_state, RunState::Halt);
  }

  #[test]
  fn interrupt() {
    let code = vec![
      0x31, 0xff, 0xc0, // LD SP, 0xc0ff
      0xfb, // EI
      0x3e, 0x1f, // LD A, 0x1f
      0xea, 0xff, 0xff, // LD (0xffff), A
      0x3e, 0x10, // LD A, 0x10
      0xea, 0x0f, 0xff, // LD (0xff0f), A
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block(); // EI will end a block
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x60);
    assert_eq!(core.memory.work_ram[0xfe], 0x00);
    assert_eq!(core.memory.work_ram[0xfd], 0x0e);
  }

  #[test]
  fn masked_interrupts() {
    let code = vec![
      0x31, 0xff, 0xc0, // LD SP, 0xc0ff
      0xfb, // EI
      0x3e, 0x03, // LD A, 0x03
      0xea, 0xff, 0xff, // LD (0xffff), A
      0x3e, 0x10, // LD A, 0x10
      0xea, 0x0f, 0xff, // LD (0xff0f), A
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block(); // EI will end a block
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x0e);
    assert_eq!(core.registers.get_sp(), 0xc0ff);
  }

  #[test]
  fn multiple_interrupts() {
    let code = vec![
      0x31, 0xff, 0xc0, // LD SP, 0xc0ff
      0xfb, // EI
      0x3e, 0x1f, // LD A, 0x1f
      0xea, 0xff, 0xff, // LD (0xffff), A
      0x3e, 0x12, // LD A, 0x12
      0xea, 0x0f, 0xff, // LD (0xff0f), A
    ];
    let mut core = Core::with_code_block(code.into_boxed_slice());
    core.run_code_block(); // EI will end a block
    core.run_code_block();
    assert_eq!(core.registers.get_ip(), 0x48);
    assert_eq!(core.memory.work_ram[0xfe], 0x00);
    assert_eq!(core.memory.work_ram[0xfd], 0x0e);
  }
}

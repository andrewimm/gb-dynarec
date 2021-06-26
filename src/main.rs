pub mod cache;
pub mod cpu;
pub mod decoder;
pub mod emitter;
pub mod emulator;
pub mod mem;

fn main() {
  // Initialize UI/Audio/Input
  
  // Load ROM, parse MMC type

  // Build and reset Dynarec Core
  /*
  let code = vec![
    0x01, 0xbb, 0xaa,
    0x21, 0x11, 0x22,
    0x2e, 0xfa,
  ];
  let mut core = emulator::Core::with_code_block(code.into_boxed_slice());
  core.run_code_block();
  */

  let code = vec![
    0x3e, 0x0d, // LD A, 0x0d
      0x1e, 0xfc, // LD E, 0xfc
      0xea, 0x00, 0xc2, // LD (0xc200), A
      0x26, 0xc2, // LD H, 0xc2
      0x2e, 0x00, // LD L, 0x00
      0x3e, 0x02, // LD A, 0x02
      0x8e, // ADC A, (HL)
  ];
  let mut core = emulator::Core::with_code_block(code.into_boxed_slice());
  core.run_code_block();
}

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
  let mut core = emulator::Core::with_code_block(code.into_boxed_slice());
  core.run_code_block();
  core.run_code_block();
}

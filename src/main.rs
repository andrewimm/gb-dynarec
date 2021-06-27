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

  let code = vec![
    0x31, 0x00, 0xc1,
    0xcc, 0x10, 0x00,
  ];
  let mut core = emulator::Core::with_code_block(code.into_boxed_slice());
  core.registers.af = 0x00f0;
  core.run_code_block();
}

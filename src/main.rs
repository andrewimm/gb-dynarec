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
    0x3e, 0x16,
    0x1e, 0x20, // LD E, 0x20
    0xbb, // CP E
  ];
  let mut core = emulator::Core::with_code_block(code.into_boxed_slice());
  core.run_code_block();
}

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
    0xc6, 0x91, // ADD A, 0x91
    0xc6, 0x93, // ADD A, 0x93
    0x27, // DAA
  ];
  let mut core = emulator::Core::with_code_block(code.into_boxed_slice());
  core.run_code_block();
}

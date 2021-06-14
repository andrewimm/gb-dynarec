pub mod cache;
pub mod cpu;
pub mod decoder;
pub mod emitter;
pub mod emulator;

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
    0x06, 0x0f, // LD B, 0x0f
      0xcb, 0x40, // BIT 0,B
      0xc3, 0x07, 0x00, // JMP 0x0007
  ];
  let mut core = emulator::Core::with_code_block(code.into_boxed_slice());
  core.run_code_block();
  println!("AF: {:X}", core.registers.get_af());
}

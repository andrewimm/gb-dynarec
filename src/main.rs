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
    0x31, 0x20, 0x44, // LD SP, 0x4420
    0x08, 0x40, 0xc0, // LD (0xc040), SP
  ];
  let mut core = emulator::Core::with_code_block(code.into_boxed_slice());
  core.run_code_block();
  println!("RAM 0xc040 = {:X}", core.memory.work_ram[0x40]);
  println!("RAM 0xc041 = {:X}", core.memory.work_ram[0x41]);
}

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
    0x26, 0xc0, // LD H, 0xc0
    0x2e, 0x03, // LD L, 0x03
    0x36, 0x58, // LD (HL), 0x58
  ];
  let mut core = emulator::Core::with_code_block(code.into_boxed_slice());
  core.run_code_block();
  println!("RAM 0xc003 = {:X}", core.memory.work_ram[0x03]);
}

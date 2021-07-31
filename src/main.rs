pub mod cache;
pub mod cpu;
pub mod cart;
pub mod debug;
pub mod decoder;
pub mod devices;
pub mod emitter;
pub mod emulator;
pub mod mem;
pub mod system;

use std::env;

fn main() {
  // Initialize UI/Audio/Input

  let mut core = match get_file_arg().and_then(load_rom) {
    Some(core) => core,
    None => fallback_core(),
  };
  
  // Build and reset Dynarec Core

  loop {
    core.run_code_block();
  }
}

fn get_file_arg() -> Option<String> {
  let mut iter = env::args();
  let _ = iter.next();
  iter.next()
}

fn load_rom(rom_file_name: String) -> Option<emulator::Core> {
  // Load ROM, parse MMC type
  let mut rom_file = {
    match system::open_rom_file(rom_file_name) {
      Ok(fd) => fd,
      Err(msg) => {
        println!("{}", msg);
        return None;
      },
    }
  };
  let header = match system::read_header(&mut rom_file) {
    Ok(head) => head,
    Err(msg) => {
      println!("{}", msg);
      return None;
    }
  };

  if !header.valid_checksum() {
    println!("ROM file is corrupt: invalid header checksum");
    return None;
  }

  println!("Loading \"{}\"", header.get_title());

  Some(emulator::Core::from_rom_file(&mut rom_file, header))
}

fn fallback_core() -> emulator::Core {
  println!("No ROM, loading fallback");
  // send "GB" over serial port
  let code = vec![
    0x3e, 0x47,
    0xe0, 0x01,
    0x3e, 0x80,
    0xe0, 0x02,
    0x3e, 0x42,
    0xe0, 0x01,
    0x3e, 0x80,
    0xe0, 0x02,
    0xc3, 0x10, 0x00,
  ];

  {
    let disassembly = debug::disassembly::disassemble(0, &code.clone().into_boxed_slice());
    for instr in disassembly.iter() {
      println!("{}", instr);
    }
  }

  emulator::Core::with_code_block(code.into_boxed_slice())
}

fn print_usage() {
  println!("No ROM file specified");
}

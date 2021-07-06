pub mod cache;
pub mod cpu;
pub mod cart;
pub mod decoder;
pub mod emitter;
pub mod emulator;
pub mod mem;
pub mod system;

use std::env;

fn main() {
  // Initialize UI/Audio/Input
  
  // Load ROM, parse MMC type
  let mut rom_file = {
    let mut iter = env::args();
    let _ = iter.next();
    let rom_file_arg = iter.next();
    let rom_file_name = match rom_file_arg {
      Some(name) => name,
      None => {
        print_usage();
        return;
      },
    };
    match system::open_rom_file(rom_file_name) {
      Ok(fd) => fd,
      Err(msg) => {
        println!("{}", msg);
        return;
      },
    }
  };
  let header = match system::read_header(&mut rom_file) {
    Ok(head) => head,
    Err(msg) => {
      println!("{}", msg);
      return;
    }
  };

  if !header.valid_checksum() {
    println!("ROM file is corrupt: invalid header checksum");
    return;
  }

  println!("Loading \"{}\"", header.get_title());

  // Build and reset Dynarec Core

  let mut core = emulator::Core::from_rom_file(&mut rom_file, header);
  core.run_code_block();
  core.run_code_block();
  core.run_code_block();
  core.run_code_block();
}

fn print_usage() {
  println!("No ROM file specified");
}

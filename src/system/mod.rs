pub mod linux;

use linux::{map_rom_file, unmap_rom_file};

use crate::cart::Header;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::mem;
use std::path::Path;
use std::string::String;

pub fn open_rom_file(name: String) -> Result<File, String> {
  let path = Path::new(&name);
  File::open(path).map_err(|_| String::from("Unable to open file"))
}

pub fn read_header(rom_file: &mut File) -> Result<Header, String> {
  let pos = rom_file.seek(SeekFrom::Start(0x100)).map_err(|_| String::from("Unable to read ROM file"))?;
  if pos != 0x100 {
    return Err(String::from("File too short. Are you sure this is a ROM file?"));
  }
  let mut header = unsafe { mem::zeroed::<Header>() };
  let read_length = mem::size_of::<Header>();
  unsafe {
    let buffer = std::slice::from_raw_parts_mut(
      &mut header as *mut Header as *mut u8,
      read_length,
    );
    rom_file.read_exact(buffer).map_err(|_| String::from("Unable to read ROM header"))?;
  }

  Ok(header)
}

pub fn get_rom_buffer(rom_file: &mut File, rom_size: usize) -> Box<[u8]> {
  map_rom_file(rom_file, rom_size)
}

pub fn drop_rom_buffer(buffer: Box<[u8]>) {
  unmap_rom_file(buffer)
}

use std::ffi::c_void;
use std::fs::File;
use std::os::windows::io::AsRawHandle;

mod bindings {
  windows::include_bindings!();
}

use bindings::{
  Windows::Win32::System::Memory::{
    CreateFileMappingA,
    MapViewOfFile,
    UnmapViewOfFile,
    FILE_MAP_READ,
    PAGE_READONLY,
  },
  Windows::Win32::Foundation::{
    HANDLE,
    PSTR,
  },
};

pub fn map_rom_file(file: &mut File, size: usize) -> Box<[u8]> {
  unsafe {
    let handle: HANDLE = CreateFileMappingA(
      HANDLE(file.as_raw_handle() as isize),
      std::ptr::null_mut(),
      PAGE_READONLY,
      0,
      0,
      PSTR::NULL,
    );
    //if handle == std::ptr::null() {
    //  panic!("Unable to create file mapping for ROM file");
    //}
    let pointer: *mut c_void = MapViewOfFile(
      handle,
      FILE_MAP_READ,
      0,
      0,
      size,
    );
    if pointer == std::ptr::null_mut() {
      panic!("Unable to create file mapping for ROM file");
    }
    Vec::from_raw_parts(pointer as *mut u8, size, size).into_boxed_slice()
  }
}

pub fn unmap_rom_file(buffer: Box<[u8]>) {
  let size = buffer.len();
  let address = Box::into_raw(buffer) as *mut () as *mut std::ffi::c_void;
  unsafe {
    UnmapViewOfFile(address);
  }
}
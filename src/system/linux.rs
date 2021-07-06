use std::ffi::c_void;
use std::fs::File;
use std::os::unix::io::AsRawFd;

pub fn map_rom_file(file: &mut File, size: usize) -> Box<[u8]> {
  unsafe {
    let pointer: *mut c_void = libc::mmap(
      std::ptr::null_mut(),
      size,
      libc::PROT_READ | libc::PROT_WRITE,
      libc::MAP_PRIVATE,
      file.as_raw_fd(),
      0, // offset
    );
    if pointer == libc::MAP_FAILED {
      panic!("Unable to mmap ROM file");
    }
    Vec::from_raw_parts(pointer as *mut u8, size, size).into_boxed_slice()
  }
}

pub fn unmap_rom_file(buffer: Box<[u8]>) {
  let size = buffer.len();
  unsafe {
    libc::munmap(
      Box::into_raw(buffer) as *mut () as *mut std::ffi::c_void,
      size,
    );
  }
}
pub struct ExecutableMemory {
  memory: Option<Box<[u8]>>,
}

impl ExecutableMemory {
  pub fn new() -> Self {
    let size: usize = 4096;
    let memory_area = unsafe {
      let pointer: *mut std::ffi::c_void = libc::mmap(
        std::ptr::null_mut(),
        size,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
        -1,
        0,
      );
      if pointer == libc::MAP_FAILED {
        panic!("Unable to mmap executable memory");
      }
      Vec::from_raw_parts(pointer as *mut u8, size, size).into_boxed_slice()
    };
    Self {
      memory: Some(memory_area),
    }
  }

  pub fn extend(&self) {

  }

  pub fn get_memory_area(&self) -> &Box<[u8]> {
    self.memory.as_ref().unwrap()
  }

  pub fn get_memory_area_mut(&mut self) -> &mut Box<[u8]> {
    self.memory.as_mut().unwrap()
  }

  pub fn make_writable(&self) {
    let size = self.memory.as_ref().unwrap().len();
    let address = self.memory.as_ref().unwrap().as_ptr() as *mut ();
    apply_protection(address, size, libc::PROT_READ | libc::PROT_WRITE);
  }

  pub fn make_executable(&self) {
    let size = self.memory.as_ref().unwrap().len();
    let address = self.memory.as_ref().unwrap().as_ptr() as *mut ();
    apply_protection(address, size, libc::PROT_READ | libc::PROT_EXEC);
  }
}

impl Drop for ExecutableMemory {
  fn drop(&mut self) {
    let memory = self.memory.take().unwrap();
    let size = memory.len();
    unsafe {
      libc::munmap(
        Box::into_raw(memory) as *mut () as *mut std::ffi::c_void,
        size,
      );
    }
  }
}

fn apply_protection(address: *mut (), size: usize, protection: i32) {
  unsafe {
    let result = libc::mprotect(
      address as *mut std::ffi::c_void,
      size,
      protection,
    );
    if result != 0 {
      panic!("Failed to modify memory protection");
    }
  }
}

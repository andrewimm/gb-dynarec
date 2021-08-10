use std::ffi::c_void;

mod bindings {
  windows::include_bindings!();
}

use bindings::{
  Windows::Win32::System::Memory::VirtualAlloc,
  Windows::Win32::System::Memory::VirtualFree,
  Windows::Win32::System::Memory::VirtualProtect,
  Windows::Win32::System::Memory::MEM_COMMIT,
  Windows::Win32::System::Memory::MEM_RELEASE,
  Windows::Win32::System::Memory::PAGE_TYPE,
  Windows::Win32::System::Memory::PAGE_EXECUTE_READ,
  Windows::Win32::System::Memory::PAGE_READWRITE,
};

pub struct ExecutableMemory {
  memory: Option<Box<[u8]>>,
}

impl ExecutableMemory {
  pub fn new() -> Self {
    let size: usize = super::INITIAL_MEMORY_SIZE;
    let memory_area = unsafe {
      let pointer: *mut c_void = VirtualAlloc(
        std::ptr::null_mut(),
        size,
        MEM_COMMIT,
        PAGE_READWRITE,
      );
      
      if pointer == std::ptr::null_mut() {
        panic!("Unable to VirtualAlloc executable memory");
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
    apply_protection(address, size, PAGE_READWRITE);
  }

  pub fn make_executable(&self) {
    let size = self.memory.as_ref().unwrap().len();
    let address = self.memory.as_ref().unwrap().as_ptr() as *mut ();
    apply_protection(address, size, PAGE_EXECUTE_READ);
  }
}

impl Drop for ExecutableMemory {
  fn drop(&mut self) {
    let memory = self.memory.take().unwrap();
    let size = memory.len();
    let address = Box::into_raw(memory) as *mut () as *mut c_void;
    unsafe {
      VirtualFree(
        address,
        size,
        MEM_RELEASE,
      );
    }
  }
}

fn apply_protection(address: *mut (), size: usize, protection: PAGE_TYPE) {
  let mut old_protect: PAGE_TYPE = PAGE_READWRITE;
  unsafe {
    VirtualProtect(
      address as *mut c_void,
      size,
      protection,
      &mut old_protect as *mut PAGE_TYPE,
    );
  }
}

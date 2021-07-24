pub struct SerialComms {
  latch: u8,
  control: u8,
}

impl SerialComms {
  pub fn new() -> Self {
    Self {
      latch: 0,
      control: 0,
    }
  }

  pub fn get_data(&self) -> u8 {
    self.latch
  }

  pub fn set_data(&mut self, value: u8) {
    self.latch = value;
  }

  pub fn get_control(&self) -> u8 {
    self.control
  }

  pub fn set_control(&mut self, value: u8) {
    use std::io::{self, Write};

    self.control = value;

    if value & 0x80 != 0 {
      let _ = io::stdout().write(&[self.latch]);
      let _ = io::stdout().flush();
    }
  }
}

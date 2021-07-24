use super::serial::SerialComms;

pub struct IO {
  pub interrupt_flag: u8,
  pub serial: Box<SerialComms>,
}

impl IO {
  pub fn new() -> Self {
    Self {
      interrupt_flag: 0,
      serial: Box::new(SerialComms::new()),
    }
  }

  pub fn set_byte(&mut self, addr: u16, value: u8) {
    match addr & 0xff {
      0x01 => self.serial.set_data(value),
      0x02 => self.serial.set_control(value),

      0x0f => self.interrupt_flag = value,
      _ => (),
    }
  }

  pub fn get_byte(&self, addr: u16) -> u8 {
    match addr & 0xff {
      0x0f => self.interrupt_flag,
      _ => 0xff,
    }
  }
}
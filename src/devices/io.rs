use super::serial::SerialComms;

pub struct IO {
  pub interrupt_flag: u8,
  pub interrupt_mask: u8,
  pub serial: Box<SerialComms>,
}

impl IO {
  pub fn new() -> Self {
    Self {
      interrupt_flag: 0,
      interrupt_mask: 0,
      serial: Box::new(SerialComms::new()),
    }
  }

  pub fn set_byte(&mut self, addr: u16, value: u8) {
    match addr & 0xff {
      0x01 => self.serial.set_data(value),
      0x02 => self.serial.set_control(value),

      0x0f => self.interrupt_flag = value & 0x1f,
      _ => (),
    }
  }

  pub fn get_byte(&self, addr: u16) -> u8 {
    match addr & 0xff {
      0x0f => self.interrupt_flag,
      _ => 0xff,
    }
  }

  pub fn get_active_interrupts(&self) -> u8 {
    self.interrupt_flag & self.interrupt_mask
  }

  /// Catch up the internal clocks of peripherals on the bus.
  /// Every time the CPU runs for a series of instructions, this should be
  /// called to keep the rest of the devices in sync.
  pub fn run_clock_cycles(&mut self, cycles: usize) {

  }
}
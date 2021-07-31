use super::interrupts::InterruptFlag;
use super::serial::SerialComms;
use super::timer::Timer;

pub struct IO {
  pub interrupt_flag: InterruptFlag,
  pub interrupt_mask: u8,
  pub serial: Box<SerialComms>,
  pub timer: Box<Timer>,
}

impl IO {
  pub fn new() -> Self {
    Self {
      interrupt_flag: InterruptFlag::empty(),
      interrupt_mask: 0,
      serial: Box::new(SerialComms::new()),
      timer: Box::new(Timer::new()),
    }
  }

  pub fn set_byte(&mut self, addr: u16, value: u8) {
    match addr & 0xff {
      0x01 => self.serial.set_data(value),
      0x02 => self.serial.set_control(value),
      0x03 => (),
      0x04 => self.timer.reset_divider(),
      0x05 => self.timer.set_counter(value),
      0x06 => self.timer.set_modulo(value),
      0x07 => {
        let flag = self.timer.set_timer_control(value);
        self.interrupt_flag |= flag;
      },

      0x0f => self.interrupt_flag = InterruptFlag::new(value & 0x1f),
      _ => (),
    }
  }

  pub fn get_byte(&self, addr: u16) -> u8 {
    match addr & 0xff {
      0x03 => 0xff,
      0x04 => self.timer.get_divider(),
      0x05 => self.timer.get_counter(),
      0x06 => self.timer.get_modulo(),
      0x07 => self.timer.get_timer_control(),

      0x0f => self.interrupt_flag.as_u8(),
      _ => 0xff,
    }
  }

  pub fn get_active_interrupts(&self) -> u8 {
    self.interrupt_flag.as_u8() & self.interrupt_mask
  }

  /// Catch up the internal clocks of peripherals on the bus.
  /// Every time the CPU runs for a series of instructions, this should be
  /// called to keep the rest of the devices in sync.
  pub fn run_clock_cycles(&mut self, cycles: usize) {
    let mut flags = self.timer.run_cycles(cycles as u32);

    self.interrupt_flag |= flags;
  }
}
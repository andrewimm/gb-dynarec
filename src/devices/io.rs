use crate::timing::ClockCycles;

use super::interrupts::InterruptFlag;
use super::joypad::Joypad;
use super::serial::SerialComms;
use super::timer::Timer;
use super::video::VideoState;

pub struct IO {
  pub interrupt_flag: InterruptFlag,
  pub interrupt_mask: u8,
  pub joypad: Box<Joypad>,
  pub serial: Box<SerialComms>,
  pub timer: Box<Timer>,
  pub video: Box<VideoState>,
}

impl IO {
  pub fn new() -> Self {
    Self {
      interrupt_flag: InterruptFlag::empty(),
      interrupt_mask: 0,
      joypad: Box::new(Joypad::new()),
      serial: Box::new(SerialComms::new()),
      timer: Box::new(Timer::new()),
      video: Box::new(VideoState::new()),
    }
  }

  pub fn set_byte(&mut self, addr: u16, value: u8) {
    match addr & 0xff {
      0x00 => self.joypad.set_value(value),
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

      0x40 => self.video.set_lcd_control(value),
      0x41 => {
        let flag = self.video.set_lcd_status(value);
        self.interrupt_flag |= flag;
      },
      0x42 => self.video.set_scroll_y(value),
      0x43 => self.video.set_scroll_x(value),
      0x44 => (),
      0x45 => {
        let flag = self.video.set_ly_compare(value);
        self.interrupt_flag |= flag;
      },

      0x47 => self.video.set_bgp(value),
      0x48 => self.video.set_obj_palette(0, value),
      0x49 => self.video.set_obj_palette(1, value),
      0x4a => self.video.set_window_y(value),
      0x4b => self.video.set_window_x(value),

      _ => (),
    }
  }

  pub fn get_byte(&self, addr: u16) -> u8 {
    match addr & 0xff {
      0x00 => self.joypad.get_value(),

      0x03 => 0xff,
      0x04 => self.timer.get_divider(),
      0x05 => self.timer.get_counter(),
      0x06 => self.timer.get_modulo(),
      0x07 => self.timer.get_timer_control(),

      // unconnected lines should be tied high
      0x0f => self.interrupt_flag.as_u8() | 0xe0,

      0x40 => self.video.get_lcd_control(),
      0x41 => self.video.get_lcd_status(),
      0x42 => self.video.get_scroll_y(),
      0x43 => self.video.get_scroll_x(),
      0x44 => self.video.get_ly(),
      0x45 => self.video.get_ly_compare(),

      0x47 => self.video.get_bgp(),

      0x48 => self.video.get_obj_palette(0),
      0x49 => self.video.get_obj_palette(1),
      0x4a => self.video.get_window_y(),
      0x4b => self.video.get_window_x(),

      _ => 0xff,
    }
  }

  pub fn get_active_interrupts(&self) -> u8 {
    self.interrupt_flag.as_u8() & self.interrupt_mask
  }

  /// Catch up the internal clocks of peripherals on the bus.
  /// Every time the CPU runs for a series of instructions, this should be
  /// called to keep the rest of the devices in sync.
  /// Since many of the devices run in terms of clock cycles, not machine (cpu)
  /// cycles, the submitted number of cycles should be 4x the number of machine
  /// cycles that have passed.
  pub fn run_clock_cycles(&mut self, cycles: ClockCycles, vram: &Box<[u8]>, oam: &Box<[u8]>) {
    let mut flags = self.timer.run_cycles(cycles);
    flags |= self.video.run_clock_cycles(cycles, vram, oam);
    flags |= self.joypad.get_interrupt();

    self.interrupt_flag |= flags;
  }
}

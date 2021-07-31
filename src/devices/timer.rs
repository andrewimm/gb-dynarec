use super::interrupts::InterruptFlag;

pub struct Timer {
  /// The timer contains a 16-bit counter, the higher 8 bits of which constitute
  /// the DIV register.
  /// To simplify handling overflows, we use unchecked adds to increase the
  /// cycle count, and always AND it with 0xffff when done.
  cycle_count: u32,
  /// The internal timer is an 8-bit counter that is incremented at a variable
  /// rate. It watches a specific bit in the cycle counter with a falling-edge
  /// detector (it triggers when the value falls from high to low). Each time
  /// that happens, the timer counter is incremented.
  counter: u8,
  /// If the timer counter overflows, it is reset to this value.
  modulo: u8,
  /// The timer is only incremented if it is enabled
  enabled_mask: u32,
  timer_clock_mask: u32,
  control_value: u8,
}

impl Timer {
  pub fn new() -> Self {
    Self {
      cycle_count: 0,
      counter: 0,
      modulo: 0,
      enabled_mask: 0,
      timer_clock_mask: 0,
      control_value: 0,
    }
  }

  pub fn reset_divider(&mut self) {
    self.cycle_count = 0;
  }

  pub fn get_divider(&self) -> u8 {
    ((self.cycle_count & 0xff00) >> 8) as u8
  }

  #[cfg(test)]
  pub fn set_divider(&mut self, value: u8) {
    self.cycle_count = (value as u32) << 8;
  }

  #[cfg(test)]
  pub fn set_cycle_count(&mut self, value: u32) {
    self.cycle_count = value;
  }

  #[cfg(test)]
  pub fn get_cycle_count(&self) -> u32 {
    self.cycle_count
  }

  pub fn set_counter(&mut self, value: u8) {
    self.counter = value;
  }

  pub fn get_counter(&self) -> u8 {
    self.counter
  }

  fn increment_counter(&mut self) -> InterruptFlag {
    if self.counter == 0xff {
      self.counter = self.modulo;
      InterruptFlag::new(1)
    } else {
      self.counter = self.counter.wrapping_add(1);
      InterruptFlag::empty()
    }
  }

  pub fn set_modulo(&mut self, value: u8) {
    self.modulo = value;
  }

  pub fn get_modulo(&self) -> u8 {
    self.modulo
  }

  /// Modify the timer control flags.
  /// Since changing a timer can trigger an interrupt, this method returns a
  /// flag value.
  pub fn set_timer_control(&mut self, flags: u8) -> InterruptFlag {
    self.control_value = flags;
    let old_masked_bit = self.cycle_count & self.timer_clock_mask & self.enabled_mask;
    self.enabled_mask = if flags & 4 != 0 {
      0xffff
    } else {
      0
    };
    self.timer_clock_mask = match flags & 3 {
      1 => { // every 16 machine cycles
        1 << 3
      },
      2 => { // every 64 machine cycles
        1 << 5
      },
      3 => { // every 256 machine cycles
        1 << 7
      },
      _ => { // every 1024 machine cycles
        1 << 9
      },
    };

    // Check if changing the mask or enabled flag triggered a falling edge
    if old_masked_bit != 0 {
      let new_masked_bit = self.cycle_count & self.timer_clock_mask & self.enabled_mask;
      if new_masked_bit == 0 {
        // This will be seen as a falling edge
        return self.increment_counter();
      }
    }

    InterruptFlag::empty()
  }

  pub fn get_timer_control(&self) -> u8 {
    self.control_value
  }

  pub fn run_cycles(&mut self, cycles: u32) -> InterruptFlag {
    if self.enabled_mask == 0 {
      // skip the edge checking
      self.cycle_count += cycles;
      self.cycle_count &= 0xffff;
      return InterruptFlag::empty();
    }
    let mut flag = InterruptFlag::empty();
    let mut to_increment = cycles;
    while to_increment > 0 {
      let mask_prev = self.cycle_count & self.timer_clock_mask;
      self.cycle_count += 1;
      let mask_new = self.cycle_count & self.timer_clock_mask;
      if mask_prev != 0 && mask_new == 0 {
        flag |= self.increment_counter();
      }
      to_increment -= 1;
    }
    self.cycle_count &= 0xffff;
    flag
  }
}

#[cfg(test)]
mod tests {
  use super::{InterruptFlag, Timer};

  #[test]
  fn timer_increment() {
    let mut timer = Timer::new();
    timer.set_timer_control(5);
    timer.run_cycles(15);
    assert_eq!(timer.get_cycle_count(), 15);
    assert_eq!(timer.get_counter(), 0);
    timer.run_cycles(1);
    assert_eq!(timer.get_counter(), 1);
  }

  #[test]
  fn timer_resolution_glitch() {
    let mut timer = Timer::new();
    timer.set_timer_control(5);
    timer.set_cycle_count(0x8);
    assert_eq!(timer.get_counter(), 0);
    timer.set_timer_control(6);
    assert_eq!(timer.get_counter(), 1);
  }

  #[test]
  fn timer_disable_glitch() {
    let mut timer = Timer::new();
    timer.set_timer_control(6);
    timer.set_cycle_count(0x3f);
    assert_eq!(timer.get_counter(), 0);
    timer.set_timer_control(2);
    assert_eq!(timer.get_counter(), 1);
  }

  #[test]
  fn timer_overflow() {
    let mut timer = Timer::new();
    timer.set_modulo(200);
    timer.set_timer_control(5);
    assert_eq!(timer.run_cycles(255 * 16), InterruptFlag::empty());
    assert_eq!(timer.get_counter(), 255);
    assert_eq!(timer.run_cycles(16), InterruptFlag::new(1));
    assert_eq!(timer.get_counter(), 200);
  }
}
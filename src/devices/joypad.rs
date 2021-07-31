use super::interrupts::InterruptFlag;

pub enum Button {
  A,
  B,
  Select,
  Start,
  Right,
  Left,
  Up,
  Down,
}

pub struct Joypad {
  action_state: u8,
  direction_state: u8,
  select_action: bool,
  select_direction: bool,
  next_interrupt: InterruptFlag,
}

impl Joypad {
  pub fn new() -> Self {
    Self {
      action_state: 0,
      direction_state: 0,
      select_action: false,
      select_direction: false,
      next_interrupt: InterruptFlag::empty(),
    }
  }

  pub fn press_button(&mut self, button: Button) {
    let prev_value = self.get_value() & 0x0f;
    match button {
      Button::A => self.action_state |= 0x01,
      Button::B => self.action_state |= 0x02,
      Button::Select => self.action_state |= 0x04,
      Button::Start => self.action_state |= 0x08,

      Button::Right => self.direction_state |= 0x01,
      Button::Left => self.direction_state |= 0x02,
      Button::Up => self.direction_state |= 0x04,
      Button::Down => self.direction_state |= 0x08,
    }
    let new_value = self.get_value() & 0x0f;

    if new_value < prev_value { // a bit went low
      self.next_interrupt = InterruptFlag::joypad();
    }
  }

  pub fn release_button(&mut self, button: Button) {
    match button {
      Button::A => self.action_state &= 0xfe,
      Button::B => self.action_state &= 0xfd,
      Button::Select => self.action_state &= 0xfb,
      Button::Start => self.action_state &= 0xf7,

      Button::Right => self.direction_state &= 0xfe,
      Button::Left => self.direction_state &= 0xfd,
      Button::Up => self.direction_state &= 0xfb,
      Button::Down => self.direction_state &= 0xf7,
    }
  }

  pub fn set_value(&mut self, value: u8) {
    let prev_value = self.get_value() & 0x0f;
    self.select_direction = value & 0x10 == 0;
    self.select_action = value & 0x20 == 0;
    let new_value = self.get_value() & 0x0f;
    if new_value < prev_value {
      self.next_interrupt = InterruptFlag::joypad();
    }
  }

  pub fn get_value(&self) -> u8 {
    let mut value = 0xc0;

    if self.select_direction {
      value |= 0x10;
      value |= self.direction_state;
    }
    if self.select_action {
      value |= 0x20;
      value |= self.action_state;
    }

    !value
  }

  pub fn get_interrupt(&mut self) -> InterruptFlag {
    std::mem::replace(&mut self.next_interrupt, InterruptFlag::empty())
  }
}

#[cfg(test)]
mod tests {
  use super::{Button, InterruptFlag, Joypad};

  #[test]
  pub fn joypad_initialize() {
    let joypad = Joypad::new();
    assert_eq!(joypad.get_value() & 0x3f, 0x3f);
  }

  #[test]
  pub fn joypad_action() {
    let mut joypad = Joypad::new();
    joypad.press_button(Button::A);
    joypad.press_button(Button::Select);
    joypad.set_value(0x20);
    assert_eq!(joypad.get_value() & 0x3f, 0x2f);
    joypad.set_value(0x10);
    assert_eq!(joypad.get_value() & 0x3f, 0x1a);
    joypad.press_button(Button::Left);
    assert_eq!(joypad.get_value() & 0x3f, 0x1a);

    joypad.release_button(Button::B);
    joypad.release_button(Button::A);
    assert_eq!(joypad.get_value() & 0x3f, 0x1b);
  }

  #[test]
  pub fn joypad_direction() {
    let mut joypad = Joypad::new();
    joypad.press_button(Button::Right);
    joypad.press_button(Button::Up);
    joypad.set_value(0x20);
    assert_eq!(joypad.get_value() & 0x3f, 0x2a);
    joypad.set_value(0x10);
    assert_eq!(joypad.get_value() & 0x3f, 0x1f);
    
    joypad.release_button(Button::Right);
    joypad.set_value(0x20);
    assert_eq!(joypad.get_value() & 0x3f, 0x2b);
  }

  #[test]
  pub fn joypad_interrupt() {
    let mut joypad = Joypad::new();
    assert_eq!(joypad.get_interrupt(), InterruptFlag::empty());
    // pushing a button does not trigger an interrupt unless a set is selected
    joypad.press_button(Button::B);
    assert_eq!(joypad.get_interrupt(), InterruptFlag::empty());
    joypad.set_value(0x20);
    assert_eq!(joypad.get_interrupt(), InterruptFlag::empty());
    // selecting the action buttons pulls bit 1 low, triggering the interrupt
    joypad.set_value(0x10);
    assert_eq!(joypad.get_interrupt(), InterruptFlag::joypad());
    // once the interrupt is recognized, it should be reset
    assert_eq!(joypad.get_interrupt(), InterruptFlag::empty());
    joypad.release_button(Button::B);
    // even if a button is pressed and then released, the interrupt still fires
    joypad.press_button(Button::B);
    joypad.release_button(Button::B);
    assert_eq!(joypad.get_interrupt(), InterruptFlag::joypad());
  }

  #[test]
  pub fn joypad_double_select_interrupt() {
    // If both action and direction are selected, pushing a button may not pull
    // a button value line low, and will fail to trigger an interrupt.
    let mut joypad = Joypad::new();
    joypad.set_value(0x00);
    joypad.press_button(Button::Start);
    assert_eq!(joypad.get_value(), 0x07);
    assert_eq!(joypad.get_interrupt(), InterruptFlag::joypad());
    joypad.press_button(Button::Down);
    assert_eq!(joypad.get_interrupt(), InterruptFlag::empty());
  }
}

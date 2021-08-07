#[derive(Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct InterruptFlag(u8);

impl InterruptFlag {
  pub fn new(value: u8) -> Self {
    Self(value)
  }

  pub fn empty() -> Self {
    Self(0)
  }

  pub fn vblank() -> Self {
    Self(1)
  }

  pub fn stat() -> Self {
    Self(2)
  }

  pub fn timer() -> Self {
    Self(4)
  }

  pub fn serial() -> Self {
    Self(8)
  }

  pub fn joypad() -> Self {
    Self(16)
  }

  pub fn as_u8(&self) -> u8 {
    self.0
  }

  pub fn clear(&mut self, flag: u8) {
    self.0 &= !flag;
  }
}

impl std::ops::BitOr for InterruptFlag {
  type Output = Self;

  fn bitor(self, rhs: Self) -> Self::Output {
    InterruptFlag(self.0 | rhs.0)
  }
}

impl std::ops::BitOrAssign for InterruptFlag {
  fn bitor_assign(&mut self, rhs: Self) {
    self.0 |= rhs.0;
  }
}

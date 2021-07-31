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

  pub fn as_u8(&self) -> u8 {
    self.0
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

/// Represents a number of raw clock cycles, the smallest unit of time for all
/// GB hardware.
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct ClockCycles(pub usize);

impl ClockCycles {
  pub fn new(cycles: usize) -> Self {
    Self(cycles)
  }

  pub fn as_usize(&self) -> usize {
    self.0
  }

  pub fn as_u32(&self) -> u32 {
    self.0 as u32
  }
}

/// Represents a number of CPU "Machine" cycles. Each Machine cycle is 4 clock
/// cycles, and the fastest CPU instructions run in a single Machine cycle.
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct MachineCycles(pub usize);

impl MachineCycles {
  pub fn new(cycles: usize) -> Self {
    Self(cycles)
  }

  pub fn to_clock_cycles(&self) -> ClockCycles {
    ClockCycles::new(self.0 * 4)
  }

  pub fn as_usize(&self) -> usize {
    self.0
  }
}

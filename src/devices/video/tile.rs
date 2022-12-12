/// Fast-interleave of two 8-bit numbers using 64-bit multiplication.
/// This only requires 11 arithmetic operations to compute the result
#[inline(always)]
pub fn interleave(low: u8, high: u8) -> u16 {
  let mut acc_high = (high as u64).wrapping_mul(0x0101010101010101u64);
  acc_high &= 0x8040201008040201u64;
  acc_high = acc_high.wrapping_mul(0x0102040810204081u64);
  acc_high >>= 48;
  acc_high &= 0xaaaa;

  let mut acc_low = (low as u64).wrapping_mul(0x0101010101010101u64);
  acc_low &= 0x8040201008040201u64;
  acc_low = acc_low.wrapping_mul(0x0102040810204081u64);
  acc_low >>= 49;
  acc_low &= 0x5555;

  (acc_high | acc_low) as u16
}

#[cfg(test)]
mod tests {
  use super::interleave;

  #[test]
  fn interleave_bits() {
    assert_eq!(interleave(0b11001001, 0b10001000), 0b1101000011000001);
    assert_eq!(interleave(1, 0), 1);
    assert_eq!(interleave(0b00000001, 0b10000000), 0b1000000000000001);
  }
}

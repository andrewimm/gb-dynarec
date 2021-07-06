use std::string::String;

#[repr(C, packed)]
pub struct Header {
  entry_point: [u8; 4],
  logo: [u8; 48],
  title: [u8; 11],
  manufacturer: [u8; 4],
  cgb_support: u8,
  licensee: [u8; 2],
  sgb_support: u8,
  cart_type: u8,
  rom_size: u8,
  ram_size: u8,
  dest_code: u8,
  licensee_deprecated: u8,
  rom_version: u8,
  header_checksum: u8,
  global_checksum: [u8; 2],
}

impl Header {
  fn as_buffer(&self) -> &[u8] {
    unsafe {
      std::slice::from_raw_parts(
        self as *const Self as *const u8,
        std::mem::size_of::<Self>(),
      )
    }
  }

  pub fn get_title(&self) -> String {
    unsafe {
      let title = std::str::from_utf8_unchecked(&self.title);
      String::from(title.trim_end_matches(std::char::from_u32_unchecked(0)))
    }
  }

  pub fn get_cart_type_string(&self) -> String {
    let inner = match self.cart_type {
      0x00 => "No MBC",
      0x01 => "MBC1",
      0x02 => "MBC1 (RAM)",
      0x03 => "MBC1 (RAM, Battery)",
      0x05 => "MBC2",
      0x11 => "MBC3",
      0x19 => "MBC5",
      _ => "Unknown",
    };
    String::from(inner)
  }

  pub fn get_rom_size_bytes(&self) -> usize {
    self.get_rom_bank_count() * 16 * 1024
  }

  pub fn get_rom_bank_count(&self) -> usize {
    match self.rom_size {
      0x00 => 2,
      0x01 => 4,
      0x02 => 8,
      0x03 => 16,
      0x04 => 32,
      0x05 => 64,
      0x06 => 128,
      0x07 => 256,
      0x08 => 512,
      0x52 => 72,
      0x53 => 80,
      0x54 => 96,
      _ => 2,
    }
  }

  pub fn get_ram_size_bytes(&self) -> usize {
    match self.ram_size {
      0x00 => 0,
      0x01 => 2 * 1024,
      0x02 => 8 * 1024,
      0x03 => 32 * 1024,
      0x04 => 128 * 1024,
      0x05 => 64 * 1024,
      _ => 0,
    }
  }

  pub fn valid_checksum(&self) -> bool {
    let buffer = self.as_buffer();
    let mut check: u8 = 0;
    for i in 0x34..0x4d {
      check = check.wrapping_sub(buffer[i]);
      check = check.wrapping_sub(1);
    }
    check == self.header_checksum
  }
}

impl std::fmt::Debug for Header {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    f.debug_struct("ROM Header")
      .field("title", &self.get_title())
      .field("type", &self.get_cart_type_string())
      .field("rom banks", &self.get_rom_bank_count())
      .finish()
  }
}

#[derive(Copy, Clone)]
pub enum MBCType {
  None,
  MBC1,
  MBC2,
  MBC3,
  MBC5,
}
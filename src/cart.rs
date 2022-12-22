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

  pub fn get_cart_type(&self) -> MBCType {
    match self.cart_type {
      0x00 => MBCType::None,
      0x01 => MBCType::MBC1,
      0x02 => MBCType::MBC1,
      0x03 => MBCType::MBC1,

      0x05 => MBCType::MBC2,

      0x11 => MBCType::MBC3,

      0x19 => MBCType::MBC5,

      _ => MBCType::Unknown,
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

  pub fn create_cart_state(&self) -> Box<dyn CartState> {
    match self.cart_type {
      0x00 => Box::new(NullCartState::new()),
      0x01 | 0x02 | 0x03 => Box::new(MBC1CartState::new()),
      
      0x11 | 0x12 | 0x13 => Box::new(MBC3CartState::new()),

      _ => panic!("Unsupported cart type"),
    }
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
  Unknown,
}

pub trait CartState {
  fn write_rom(&mut self, addr: u16, value: u8) {
  }

  fn get_rom_bank(&self) -> usize {
    1
  }

  fn get_ram_bank(&self) -> usize {
    0
  }

  fn get_ram_override(&self, addr: u16) -> Option<u8> {
    None
  }
}

pub struct NullCartState {
}

impl NullCartState {
  pub fn new() -> Self {
    Self {}
  }
}

impl CartState for NullCartState {}

pub struct MBC1CartState {
  rom_bank: usize,
  ram_bank: usize,
  ram_enabled: bool,
  select_ram: bool,
}

impl MBC1CartState {
  fn new() -> Self {
    MBC1CartState {
      rom_bank: 1,
      ram_bank: 0,
      ram_enabled: false,
      select_ram: false,
    }
  }
}

impl CartState for MBC1CartState {
  fn write_rom(&mut self, addr: u16, value: u8) {
    if addr < 0x2000 {
      let enable = (value & 0x0a) == 0x0a;
      self.ram_enabled = enable;
    } else if addr < 0x4000 {
      self.rom_bank = (value & 0x1f) as usize;
    } else if addr < 0x6000 {
      self.ram_bank = (value & 0x03) as usize;
    } else {
      self.select_ram = (value & 1) == 1;
    }
  }

  fn get_rom_bank(&self) -> usize {
    if self.select_ram {
      self.rom_bank
    } else {
      let bank_high = self.ram_bank << 5;
      let mut bank = self.rom_bank;
      if bank == 0 {
        bank = 1;
      }
      bank |= bank_high;
      bank
    }
  }

  fn get_ram_bank(&self) -> usize {
    if self.select_ram {
      self.ram_bank
    } else {
      0
    }
  }

  fn get_ram_override(&self, addr: u16) -> Option<u8> {
    if self.ram_enabled {
      None
    } else {
      Some(0xff)
    }
  }
}

pub struct MBC3CartState {
  rom_bank: usize,
  ram_bank: usize,
  ram_enabled: bool,
}

impl MBC3CartState {
  pub fn new() -> Self {
    Self {
      rom_bank: 1,
      ram_bank: 0,
      ram_enabled: false,
    }
  }
}

impl CartState for MBC3CartState {
  fn write_rom(&mut self, addr: u16, value: u8) {
    if addr < 0x2000 {
      self.ram_enabled = value & 0x0a == 0x0a;
    } else if addr < 0x4000 {
      self.rom_bank = value as usize & 0x7f;
    } else if addr < 0x6000 {
      if value < 0x04 {
        self.ram_bank = value as usize;
      } else {
        // set clock register
      }
    } else {
      // latch clock data
    }
  }

  fn get_rom_bank(&self) -> usize {
    let mut bank = self.rom_bank;
    if bank == 0 {
      bank = 1;
    }
    bank
  }

  fn get_ram_bank(&self) -> usize {
    self.ram_bank
  }
}

use std::collections::BTreeMap;

/// CodeBlock is a simple pointer to a region of JIT executable memory that
/// contains translated code
pub struct CodeBlock {
  pub offset: usize,
  pub length: usize,
  pub bytes_translated: usize,
}

pub struct MemoryLocation {
  bank: u16,
  address: u16,
}

impl MemoryLocation {
  pub fn new(bank: u16, address: u16) -> Self {
    Self {
      bank,
      address,
    }
  }

  pub fn as_u32(&self) -> u32 {
    ((self.bank as u32) << 16)
      | (self.address as u32)
  }

  pub fn from_u32(value: u32) -> Self {
    let bank = (value >> 16) as u16;
    let address = value as u16;
    Self {
      bank,
      address,
    }
  }
}

/// A CacheRegion is a smaller cache for a specific segment of the GB memory
/// map. Because of banking, a single address on the GB bus can point to
/// different blocks of code at different times.
/// Creating a separate cache for each banked region of memory makes it easier
/// to perform lookup for a specific GB address.
pub struct CacheRegion {
  cache: BTreeMap<u32, CodeBlock>,
  current_bank: u16,
}

impl CacheRegion {
  pub fn new(current_bank: u16) -> Self {
    Self {
      cache: BTreeMap::new(),
      current_bank,
    }
  }

  pub fn insert(&mut self, address: u16, block: CodeBlock) {
    let location = MemoryLocation::new(self.current_bank, address);
    let key = location.as_u32();
    self.cache.insert(key, block);
  }

  pub fn get(&self, address: u16) -> Option<&CodeBlock> {
    let location = MemoryLocation::new(self.current_bank, address);
    let key = location.as_u32();
    self.cache.get(&key)
  }

  pub fn invalidate(&mut self, address: u16) -> Option<CodeBlock> {
    let location = MemoryLocation::new(self.current_bank, address);
    let key = location.as_u32();
    self.cache.remove(&key)
  }

  pub fn invalidate_containing(&mut self, address: u16) -> Option<CodeBlock> {
    let mut found = None;
    for (key, block) in self.cache.iter() {
      let ip = MemoryLocation::from_u32(*key).address;
      let length = block.bytes_translated as u16;
      if address >= ip && address < ip + length {
        found = Some(*key);
        break;
      }
    }
    found.and_then(|key| self.cache.remove(&key))
  }

  pub fn set_bank(&mut self, bank: u16) {
    self.current_bank = bank;
  }
}

/// CachedBlocks stores individual lookup caches for each region of memory that
/// could reasonably store code.
pub struct CachedBlocks {
  rom_low: CacheRegion,   // 0x0000 - 0x3fff
  rom_high: CacheRegion,  // 0x3fff - 0x7fff
  pub cart_ram: CacheRegion,  // 0xa000 - 0xbfff
  pub wram_low: CacheRegion,  // 0xc000 - 0xcfff
  pub wram_high: CacheRegion, // 0xd000 - 0xdfff
  pub high_ram: CacheRegion,  // 0xff80 - 0xfffe
}

impl CachedBlocks {
  pub fn new() -> Self {
    Self {
      rom_low: CacheRegion::new(0),
      rom_high: CacheRegion::new(1),
      cart_ram: CacheRegion::new(0),
      wram_low: CacheRegion::new(0),
      wram_high: CacheRegion::new(1),
      high_ram: CacheRegion::new(0),
    }
  }

  pub fn get_region(&self, addr: u16) -> Option<&CacheRegion> {
    if addr < 0x4000 {
      return Some(&self.rom_low);
    }
    if addr < 0x8000 {
      return Some(&self.rom_high);
    }
    if addr < 0xa000 {
      return None;
    }
    if addr < 0xc000 {
      return Some(&self.cart_ram);
    }
    if addr < 0xd000 {
      return Some(&self.wram_low);
    }
    if addr < 0xe000 {
      return Some(&self.wram_high);
    }
    if addr < 0xff80 {
      return None;
    }
    if addr != 0xffff {
      return Some(&self.high_ram);
    }
    None
  }

  pub fn get_region_mut(&mut self, addr: u16) -> Option<&mut CacheRegion> {
    if addr < 0x4000 {
      return Some(&mut self.rom_low);
    }
    if addr < 0x8000 {
      return Some(&mut self.rom_high);
    }
    if addr < 0xa000 {
      return None;
    }
    if addr < 0xc000 {
      return Some(&mut self.cart_ram);
    }
    if addr < 0xd000 {
      return Some(&mut self.wram_low);
    }
    if addr < 0xe000 {
      return Some(&mut self.wram_high);
    }
    if addr < 0xff80 {
      return None;
    }
    if addr != 0xffff {
      return Some(&mut self.high_ram);
    }
    None
  }
}

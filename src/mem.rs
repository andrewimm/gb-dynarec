pub struct MemoryAreas {
  pub rom: Box<[u8]>,
  pub video_ram: Box<[u8]>,
  pub cart_ram: Box<[u8]>,
  pub work_ram: Box<[u8]>,

  pub rom_bank: usize,
  pub vram_bank: usize,
  pub cram_bank: usize,
  pub wram_bank: usize,
}

impl MemoryAreas {
  pub fn with_rom(rom: Box<[u8]>) -> Self {
    let mut work_ram = Vec::<u8>::with_capacity(0x1000);
    for _ in 0..0x1000 {
      work_ram.push(0);
    }
    Self {
      rom,
      video_ram: vec![].into_boxed_slice(),
      cart_ram: vec![].into_boxed_slice(),
      work_ram: work_ram.into_boxed_slice(),

      rom_bank: 1,
      vram_bank: 0,
      cram_bank: 0,
      wram_bank: 1,
    }
  }

  pub fn as_ptr(&self) -> *const Self {
    self as *const Self
  }
}

pub extern "sysv64" fn memory_read_byte(areas: *const MemoryAreas, addr: u16) -> u8 {
  let memory_areas: &MemoryAreas = unsafe { &*areas };
  if addr < 0x4000 { // ROM Bank 0
    return memory_areas.rom[addr as usize];
  }
  if addr < 0x8000 { // ROM Bank NN
    let offset = addr as usize & 0x3fff;
    return memory_areas.rom[0x4000 * memory_areas.rom_bank + offset];
  }
  if addr < 0xa000 { // VRAM
    return 0;
  }
  if addr < 0xc000 { // Cart RAM

  }
  if addr < 0xd000 { // Work RAM Bank 0
    let offset = addr as usize & 0xfff;
    return memory_areas.work_ram[offset];
  }
  if addr < 0xe000 { // Work RAM Bank NN
    return 0;
  }
  // Mirror and I/O
  0
}

pub extern "sysv64" fn memory_write_byte(areas: *mut MemoryAreas, addr: u16, value: u8) {
  let memory_areas: &mut MemoryAreas = unsafe { &mut *areas };
  if addr < 0x4000 { // ROM Bank 0
    return;
  }
  if addr < 0x8000 { // ROM Bank NN
    return;
  }
  if addr < 0xa000 { // VRAM
    let offset = addr as usize & 0x1fff;
    memory_areas.video_ram[0x2000 * memory_areas.vram_bank + offset] = value;
    return;
  }
  if addr < 0xc000 { // Cart RAM

  }
  if addr < 0xd000 { // Work RAM Bank 0
    let offset = addr as usize & 0xfff;
    memory_areas.work_ram[offset] = value;
    return;
  }
  if addr < 0xe000 { // Work RAM Bank NN
    let offset = addr as usize & 0xfff;
    memory_areas.work_ram[0x1000 * memory_areas.wram_bank + offset] = value;
    return;
  }
  // Mirror and I/O
}

pub extern "sysv64" fn memory_write_word(areas: *mut MemoryAreas, addr: u16, value: u16) {
  let low = (value & 0xff) as u8;
  let high = (value >> 8) as u8;
  memory_write_byte(areas, addr, low);
  memory_write_byte(areas, addr + 1, high);
}

pub extern "sysv64" fn memory_read_word(areas: *mut MemoryAreas, addr: u16) -> u16 {
  let low = memory_read_byte(areas, addr) as u16;
  let high = memory_read_byte(areas, addr + 1) as u16;
  (high << 8) | low
}

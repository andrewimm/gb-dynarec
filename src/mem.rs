use crate::cart::Header;
use crate::devices::io::IO;
use std::fs::File;

pub struct MemoryAreas {
  pub rom: Box<[u8]>,
  pub video_ram: Box<[u8]>,
  pub cart_ram: Box<[u8]>,
  pub work_ram: Box<[u8]>,
  pub high_ram: Box<[u8]>,

  pub rom_bank: usize,
  pub vram_bank: usize,
  pub cram_bank: usize,
  pub wram_bank: usize,

  pub wram_dirty: bool,
  pub wram_dirty_flags: Box<[u64; 128]>,
  pub hram_dirty: bool,
  pub hram_dirty_flags: Box<[u64; 2]>,

  pub io: IO,

  rom_mapped: bool,
}

impl MemoryAreas {
  pub fn with_rom(rom_code: Box<[u8]>) -> Self {
    let mut rom = Vec::<u8>::with_capacity(0x4000);
    for i in 0..0x4000 {
      if i < rom_code.len() {
        rom.push(rom_code[i]);
      } else if i == rom_code.len() {
        rom.push(0x76);
      } else {
        rom.push(0);
      }
    }
    let mut work_ram = Vec::<u8>::with_capacity(0x1000);
    for _ in 0..0x1000 {
      work_ram.push(0);
    }
    let mut video_ram = Vec::<u8>::with_capacity(0x2000);
    for _ in 0..0x2000 {
      video_ram.push(0);
    }

    let wram_dirty_flags = Box::new([0; 128]);

    Self {
      rom: rom.into_boxed_slice(),
      video_ram: video_ram.into_boxed_slice(),
      cart_ram: vec![].into_boxed_slice(),
      work_ram: work_ram.into_boxed_slice(),
      high_ram: vec![].into_boxed_slice(),

      rom_bank: 1,
      vram_bank: 0,
      cram_bank: 0,
      wram_bank: 1,

      wram_dirty: false,
      wram_dirty_flags,
      hram_dirty: false,
      hram_dirty_flags: Box::new([0; 2]),

      io: IO::new(),

      rom_mapped: false,
    }
  }

  pub fn with_rom_file(rom_file: &mut File, header: &Header) -> Self {
    let rom_size = header.get_rom_size_bytes();
    let video_ram_size = 8 * 1024; // 8KB for DMB, 16KB for CGB
    let cart_ram_size = header.get_ram_size_bytes();
    let work_ram_size = 8 * 1024; // 8KB for DMG, 32KB for CGB

    let rom = crate::system::get_rom_buffer(rom_file, rom_size);
    let video_ram = create_buffer(video_ram_size);
    let cart_ram = create_buffer(cart_ram_size);
    let work_ram = create_buffer(work_ram_size);
    let high_ram = create_buffer(127);

    Self {
      rom,
      video_ram,
      cart_ram,
      work_ram,
      high_ram,
      rom_bank: 1,
      vram_bank: 0,
      cram_bank: 0,
      wram_bank: 1,

      wram_dirty: false,
      wram_dirty_flags: Box::new([0; 128]),
      hram_dirty: false,
      hram_dirty_flags: Box::new([0; 2]),

      io: IO::new(),

      rom_mapped: true,
    }
  }

  pub fn as_ptr(&self) -> *const Self {
    self as *const Self
  }

  pub fn run_clock_cycles(&mut self, cycles: usize) {
    self.io.run_clock_cycles(cycles, &self.video_ram);
  }
}

impl Drop for MemoryAreas {
  fn drop(&mut self) {
    if !self.rom_mapped {
      return;
    }
    let reset = vec![0xc3, 0x00, 0x00]; // JP 0x0000, infinite loop
    let old_rom = std::mem::replace(&mut self.rom, reset.into_boxed_slice());
    crate::system::drop_rom_buffer(old_rom);
  }
}

pub fn get_executable_memory_slice<'s>(start: usize, mem_ptr: *const MemoryAreas) -> &'s [u8] {
  let mem = unsafe { &*mem_ptr };
  match start {
    0x0000..=0x3fff => &mem.rom[start..0x4000],
    0x4000..=0x7fff => {
      let bank_start = mem.rom_bank * 0x4000;
      let bank_end = bank_start + 0x4000;
      let offset = (start & 0x3fff) + bank_start;
      &mem.rom[offset..bank_end]
    },
    0xc000..=0xcfff => &mem.work_ram[(start & 0xfff)..0x1000],
    0xd000..=0xdfff => {
      let bank_start = mem.wram_bank * 0x1000;
      let bank_end = bank_start + 0x1000;
      let offset = (start & 0xfff) + bank_start;
      &mem.work_ram[offset..bank_end]
    },
    0xff80..=0xfffe => &mem.high_ram[(start & 0x7f)..],
    _ => panic!("TRIED TO EXECUTE {:X}", start),
  }
}

fn create_buffer(size: usize) -> Box<[u8]> {
  let mut buffer = Vec::<u8>::with_capacity(size);
  for _ in 0..size {
    buffer.push(0);
  }
  buffer.into_boxed_slice()
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
    let offset = addr as usize & 0x1fff;
    return memory_areas.video_ram[offset];
  }
  if addr < 0xc000 { // Cart RAM
    panic!("Cart RAM not supported yet");
  }
  if addr < 0xd000 { // Work RAM Bank 0
    let offset = addr as usize & 0xfff;
    return memory_areas.work_ram[offset];
  }
  if addr < 0xe000 { // Work RAM Bank NN
    let offset = addr as usize & 0xfff;
    return memory_areas.work_ram[0x1000 * memory_areas.wram_bank + offset];
  }
  if addr < 0xfe00 { // Mirror
    return 0;
  }
  if addr < 0xfea0 { // OAM
    return 0;
  }
  if addr < 0xff00 { // unused
    return 0;
  }
  if addr < 0xff80 { // I/O
    return memory_areas.io.get_byte(addr);
  }
  if addr == 0xffff { // Interrupt Mask
    return memory_areas.io.interrupt_mask;
  }
  // High RAM
  memory_areas.high_ram[addr as usize & 0x7f]
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
    panic!("Cart RAM not supported yet");
  }
  if addr < 0xd000 { // Work RAM Bank 0
    let offset = addr as usize & 0xfff;
    memory_areas.work_ram[offset] = value;
    mark_wram_dirty(offset, &mut memory_areas.wram_dirty_flags);
    memory_areas.wram_dirty = true;
    return;
  }
  if addr < 0xe000 { // Work RAM Bank NN
    let offset = addr as usize & 0xfff;
    let index = 0x1000 * memory_areas.wram_bank + offset;
    memory_areas.work_ram[index] = value;
    mark_wram_dirty(index, &mut memory_areas.wram_dirty_flags);
    memory_areas.wram_dirty = true;
    return;
  }
  if addr < 0xfe00 { // Mirror
    return;
  }
  if addr < 0xfea0 { // OAM
    return;
  }
  if addr < 0xff00 { // unused
    return;
  }
  if addr < 0xff80 { // I/O
    memory_areas.io.set_byte(addr, value);
    return;
  }
  if addr == 0xffff { // Interrupt Mask
    memory_areas.io.interrupt_mask = value & 0x1f;
    return;
  }
  {
    // High RAM
    let index = addr as usize & 0x7f;
    memory_areas.high_ram[index] = value;
    mark_hram_dirty(index, &mut memory_areas.hram_dirty_flags);
    memory_areas.hram_dirty = true;
  }
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

extern "sysv64" fn mark_wram_dirty(index: usize, dirty_wram: &mut [u64; 128]) {
  let dirty_index = index / 64;
  let dirty_offset = DIRTY_OFFSETS[index & 63];
  dirty_wram[dirty_index] |= dirty_offset;
}

extern "sysv64" fn mark_hram_dirty(index: usize, dirty_hram: &mut [u64; 2]) {
  let dirty_index = index / 64;
  let dirty_offset = DIRTY_OFFSETS[index & 63];
  dirty_hram[dirty_index] |= dirty_offset;
}

const DIRTY_OFFSETS: [u64; 64] = [
  0x1, 0x2, 0x4, 0x8,
  0x10, 0x20, 0x40, 0x80,
  0x100, 0x200, 0x400, 0x800,
  0x1000, 0x2000, 0x4000, 0x8000,
  0x10000, 0x20000, 0x40000, 0x80000,
  0x100000, 0x200000, 0x400000, 0x800000,
  0x1000000, 0x2000000, 0x4000000, 0x8000000,
  0x10000000, 0x20000000, 0x40000000, 0x80000000,
  0x100000000, 0x200000000, 0x400000000, 0x800000000,
  0x1000000000, 0x2000000000, 0x4000000000, 0x8000000000,
  0x10000000000, 0x20000000000, 0x40000000000, 0x80000000000,
  0x100000000000, 0x200000000000, 0x400000000000, 0x800000000000,
  0x1000000000000, 0x2000000000000, 0x4000000000000, 0x8000000000000,
  0x10000000000000, 0x20000000000000, 0x40000000000000, 0x80000000000000,
  0x100000000000000, 0x200000000000000, 0x400000000000000, 0x800000000000000,
  0x1000000000000000, 0x2000000000000000, 0x4000000000000000, 0x8000000000000000,
];

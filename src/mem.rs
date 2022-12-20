use crate::cart::{CartState, Header, NullCartState};
use crate::devices::io::IO;
use crate::timing::ClockCycles;
use std::fs::File;

pub struct MemoryAreas {
  pub rom: Box<[u8]>,
  pub cart_state: Box<dyn CartState>,
  pub video_ram: Box<[u8]>,
  pub cart_ram: Box<[u8]>,
  pub work_ram: Box<[u8]>,
  pub oam_ram: Box<[u8]>,
  pub high_ram: Box<[u8]>,

  pub vram_bank: usize,
  pub wram_bank: usize,

  pub io: IO,

  pub oam_dma: Option<DMAState>,

  rom_mapped: bool,
}

/// Stores the state of an active DMA procedure
#[derive(Copy, Clone)]
pub struct DMAState {
  // Location to begin copying memory from, aligned to the nearest 0x100
  source: usize,
  // OAM DMA is 0xa0 bytes long. This stores the offset to be copied next.
  current_offset: u8,
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
        rom.push(0xff);
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

    Self {
      rom: rom.into_boxed_slice(),
      cart_state: Box::new(NullCartState::new()),
      video_ram: video_ram.into_boxed_slice(),
      cart_ram: vec![].into_boxed_slice(),
      work_ram: work_ram.into_boxed_slice(),
      oam_ram: create_buffer(0xa0),
      high_ram: create_buffer(127),

      vram_bank: 0,
      wram_bank: 1,

      io: IO::new(),

      oam_dma: None,

      rom_mapped: false,
    }
  }

  pub fn with_rom_file(rom_file: &mut File, header: &Header) -> Self {
    let cart_state = header.create_cart_state();
    let rom_size = header.get_rom_size_bytes();
    let video_ram_size = 8 * 1024; // 8KB for DMB, 16KB for CGB
    let cart_ram_size = header.get_ram_size_bytes();
    let work_ram_size = 8 * 1024; // 8KB for DMG, 32KB for CGB

    let rom = crate::system::get_rom_buffer(rom_file, rom_size);
    let video_ram = create_buffer(video_ram_size);
    let cart_ram = create_buffer(cart_ram_size);
    let work_ram = create_buffer(work_ram_size);
    let oam_ram = create_buffer(0xa0);
    let high_ram = create_buffer(127);

    Self {
      rom,
      cart_state,
      video_ram,
      cart_ram,
      work_ram,
      oam_ram,
      high_ram,
      vram_bank: 0,
      wram_bank: 1,

      io: IO::new(),
      oam_dma: None,

      rom_mapped: true,
    }
  }

  pub fn as_ptr(&self) -> *const Self {
    self as *const Self
  }

  pub fn get_rom_bank(&self) -> usize {
    self.cart_state.get_rom_bank()
  }

  pub fn run_clock_cycles(&mut self, cycles: ClockCycles) {
    // If a DMA is currently active, it updates with the rest of the memory bus
    // One byte is copied on each machine cycle. This will copy at most that
    // many bytes (or fewer, if the DMA completes before then).
    if let Some(dma) = self.oam_dma {
      let source = dma.source;
      let mut current_offset = dma.current_offset as usize;

      let bytes_remaining = 0xa0 - current_offset;
      let cycles_to_copy = cycles.as_usize() / 4;
      let mut bytes_to_copy = bytes_remaining.min(cycles_to_copy);

      while bytes_to_copy > 0 {
        // copy DMA byte
        let source = source + current_offset;

        let value = memory_read_byte(self as *mut MemoryAreas, source as u16);
        let dest = 0xfe00 + current_offset as u16;
        memory_write_byte(self as *mut MemoryAreas, dest, value);

        bytes_to_copy -= 1;
        current_offset += 1;
      }
      if current_offset < 0xa0 {
        self.oam_dma = Some(
          DMAState {
            source,
            current_offset: current_offset as u8,
          }
        );
      } else {
        self.oam_dma = None;
      }
    }

    self.io.run_clock_cycles(cycles, &self.video_ram, &self.oam_ram);
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
      let bank_start = mem.cart_state.get_rom_bank() * 0x4000;
      let bank_end = bank_start + 0x4000;
      let offset = (start & 0x3fff) + bank_start;
      &mem.rom[offset..bank_end]
    },
    0xc000..=0xcfff | 0xe000..=0xefff => &mem.work_ram[(start & 0xfff)..0x1000],
    0xd000..=0xdfff | 0xf000..=0xfe9f => {
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

#[inline(never)]
pub extern "sysv64" fn memory_read_byte(areas: *const MemoryAreas, addr: u16) -> u8 {
  let memory_areas: &MemoryAreas = unsafe { &*areas };
  if addr < 0x4000 { // ROM Bank 0
    return memory_areas.rom[addr as usize];
  }
  if addr < 0x8000 { // ROM Bank NN
    let offset = addr as usize & 0x3fff;
    return memory_areas.rom[0x4000 * memory_areas.cart_state.get_rom_bank() + offset];
  }
  if addr < 0xa000 { // VRAM
    let offset = addr as usize & 0x1fff;
    return memory_areas.video_ram[offset];
  }
  if addr < 0xc000 { // Cart RAM
    let offset = addr as usize & 0x1fff;
    return memory_areas.cart_ram[0x2000 * memory_areas.cart_state.get_ram_bank() + offset];
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
    let offset = addr as usize & 0xff;
    return memory_areas.oam_ram[offset];
  }
  if addr < 0xff00 { // unused
    return 0;
  }
  if addr < 0xff80 { // I/O
    if addr == 0xff46 {
      // TODO: OAM should return last written value
    } else {
      return memory_areas.io.get_byte(addr);
    }
  }
  if addr == 0xffff { // Interrupt Mask
    return memory_areas.io.interrupt_mask;
  }
  // High RAM
  memory_areas.high_ram[addr as usize & 0x7f]
}

#[inline(never)]
pub extern "sysv64" fn memory_write_byte(areas: *mut MemoryAreas, addr: u16, value: u8) {
  let memory_areas: &mut MemoryAreas = unsafe { &mut *areas };
  if addr < 0x8000 { // ROM Banks
    memory_areas.cart_state.write_rom(addr, value);
    return;
  }
  if addr < 0xa000 { // VRAM
    let offset = addr as usize & 0x1fff;
    memory_areas.video_ram[0x2000 * memory_areas.vram_bank + offset] = value;
    return;
  }
  if addr < 0xc000 { // Cart RAM
    let offset = addr as usize & 0x1fff;
    memory_areas.video_ram[0x2000 * memory_areas.cart_state.get_ram_bank() + offset] = value;
    return;
  }
  if addr < 0xd000 { // Work RAM Bank 0
    let offset = addr as usize & 0xfff;
    memory_areas.work_ram[offset] = value;
    return;
  }
  if addr < 0xe000 { // Work RAM Bank NN
    let offset = addr as usize & 0xfff;
    let index = 0x1000 * memory_areas.wram_bank + offset;
    memory_areas.work_ram[index] = value;
    return;
  }
  if addr < 0xfe00 { // Mirror
    return;
  }
  if addr < 0xfea0 { // OAM
    let offset = addr as usize & 0xff;
    memory_areas.oam_ram[offset] = value;
    return;
  }
  if addr < 0xff00 { // unused
    return;
  }
  if addr < 0xff80 { // I/O
    if addr == 0xff46 {
      let source = (value as usize) << 8;
      memory_areas.oam_dma = Some(
        DMAState {
          source,
          current_offset: 0,
        }
      );
    } else {
      memory_areas.io.set_byte(addr, value);
    }
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
  }
}

#[inline(never)]
pub extern "sysv64" fn memory_write_word(areas: *mut MemoryAreas, addr: u16, value: u16) {
  let low = (value & 0xff) as u8;
  let high = (value >> 8) as u8;
  memory_write_byte(areas, addr, low);
  memory_write_byte(areas, addr + 1, high);
}

#[inline(never)]
pub extern "sysv64" fn memory_read_word(areas: *mut MemoryAreas, addr: u16) -> u16 {
  let low = memory_read_byte(areas, addr) as u16;
  let high = memory_read_byte(areas, addr + 1) as u16;
  (high << 8) | low
}

pub fn can_dynarec(addr: usize) -> bool {
  addr < 0x8000
}


pub mod lcd;
pub mod tile;

use std::u8;

use lcd::LCD;
use crate::timing::ClockCycles;

use super::interrupts::InterruptFlag;

const SHADES: [u8; 4] = [255, 170, 85, 0];

struct ObjectAttributes {
  pub palette: u8,
  pub x_coord: u8,
  pub row_data: u16,
  pub has_priority: bool,
}

pub struct VideoState {
  lcd: LCD,
  tile_address_offset: usize,
  first_tile_offset: usize,
  bg_map_offset: usize,
  window_map_offset: usize,
  object_double_height: bool,
  object_enabled: bool,
  window_enabled: bool,
  bg_window_enabled: bool,
  lcd_control_value: u8,
  ly_compare: u8,
  interrupt_on_lyc: bool,
  interrupt_on_mode_2: bool,
  interrupt_on_mode_1: bool,
  interrupt_on_mode_0: bool,
  bg_palette: [u8; 4],
  bg_palette_value: u8,
  object_palettes: [u8; 4 * 8],
  object_palette_values: [u8; 8],
  scroll_x: u8,
  scroll_y: u8,
  window_x: u8,
  window_y: u8,

  current_mode: u8,
  current_mode_dots: usize,
  current_line: u8,
  next_cached_tile_x: usize,
  current_tile_cache: u16,
  /// At the beginning of each line, the objects for that line are pre-cached.
  /// The pixels are drawn into this buffer, where each byte represents a
  /// single object pixel in the following 8-bit format:
  ///
  ///   7         6          5        4 3 2     1 0
  /// -------------------------------------------------------
  /// | present | priority | unused | palette | color index |
  ///
  /// A `present` pixel should not be overdrawn by another object
  /// `priority` determines whether the sprite appears above or behind the BG
  /// `palette` selects the color palette (0-7 for CGB, 0-1 for DMG)
  /// `color index` determines which pixel is selected from the palette
  /// 
  /// To simplify drawing sprites that are partially off the screen, an
  /// additional 8 pixels are added before and after the visible buffer.
  object_line_cache: [u8; 176],
  current_obj_line_cache_pixel: usize,
  current_window_line: Option<usize>,
}

impl VideoState {
  pub fn new() -> Self {
    Self {
      lcd: LCD::new(),
      tile_address_offset: 0,
      first_tile_offset: 0,
      bg_map_offset: 0x1800,
      window_map_offset: 0x1800,
      object_double_height: false,
      object_enabled: false,
      window_enabled: false,
      bg_window_enabled: false,
      lcd_control_value: 0,
      ly_compare: 0,
      interrupt_on_lyc: false,
      interrupt_on_mode_2: false,
      interrupt_on_mode_1: false,
      interrupt_on_mode_0: false,
      bg_palette: [0; 4],
      bg_palette_value: 0,
      object_palettes: [0; 4 * 8],
      object_palette_values: [0; 8],
      scroll_x: 0,
      scroll_y: 0,
      window_x: 0,
      window_y: 0,

      // start at the beginning of a vblank
      current_mode: 1,
      current_mode_dots: 0,
      current_line: 144,
      next_cached_tile_x: 0,
      current_tile_cache: 0,
      object_line_cache: [0; 176],
      current_obj_line_cache_pixel: 0,
      current_window_line: None,
    }
  }

  pub fn get_current_mode(&self) -> u8 {
    self.current_mode
  }

  pub fn set_lcd_control(&mut self, value: u8) {
    self.lcd.set_enabled(value & 0x80 != 0);
    self.window_map_offset = if value & 0x40 == 0 {
      0x1800
    } else {
      0x1c00
    };
    self.window_enabled = value & 0x20 == 0x20;
    let (tile_address_offset, first_tile_offset) = if value & 0x10 == 0 {
      (0x800, 0x800)
    } else {
      (0, 0)
    };
    self.tile_address_offset = tile_address_offset;
    self.first_tile_offset = first_tile_offset;
    self.bg_map_offset = if value & 0x08 == 0 {
      0x1800
    } else {
      0x1c00
    };
    self.object_double_height = value & 0x04 == 0x04;
    self.object_enabled = value & 0x02 == 0x02;
    self.bg_window_enabled = value & 0x01 == 0x01;

    self.lcd_control_value = value;
  }

  pub fn get_lcd_control(&self) -> u8 {
    self.lcd_control_value
  }

  pub fn set_lcd_status(&mut self, value: u8) -> InterruptFlag {
    self.interrupt_on_lyc = value & 0x40 != 0;
    self.interrupt_on_mode_2 = value & 0x20 != 0;
    self.interrupt_on_mode_1 = value & 0x10 != 0;
    self.interrupt_on_mode_0 = value & 0x08 != 0;
    self.check_current_line()
  }

  pub fn get_lcd_status(&self) -> u8 {
    let mut status = 0;
    if self.interrupt_on_lyc {
      status |= 0x40;
    }
    if self.interrupt_on_mode_2 {
      status |= 0x20;
    }
    if self.interrupt_on_mode_1 {
      status |= 0x10;
    }
    if self.interrupt_on_mode_0 {
      status |= 0x08;
    }
    if self.ly_compare == self.current_line {
      status |= 4;
    }
    // set the lower two bits to the current mode
    status |= self.current_mode;
    status
  }

  pub fn set_scroll_x(&mut self, value: u8) {
    self.scroll_x = value;
  }

  pub fn get_scroll_x(&self) -> u8 {
    self.scroll_x
  }

  pub fn set_scroll_y(&mut self, value: u8) {
    self.scroll_y = value;
  }

  pub fn get_scroll_y(&self) -> u8 {
    self.scroll_y
  }

  pub fn get_ly(&self) -> u8 {
    self.current_line
  }

  pub fn set_window_x(&mut self, value: u8) {
    self.window_x = value;
  }

  pub fn get_window_x(&self) -> u8 {
    self.window_x
  }

  pub fn set_window_y(&mut self, value: u8) {
    self.window_y = value;
  }

  pub fn get_window_y(&self) -> u8 {
    self.window_y
  }

  pub fn set_ly_compare(&mut self, value: u8) -> InterruptFlag {
    self.ly_compare = value;
    self.check_current_line()
  }

  pub fn get_ly_compare(&self) -> u8 {
    self.ly_compare
  }

  /// Set the Background Palette
  /// The palette (for a standard monochrome GB) is an 8-bit number, where
  /// every 2 bits represent the value of a color. Color 0 is found at the
  /// lowest 2 bits, color 1 at the next 2 bits, and so on.
  pub fn set_bgp(&mut self, value: u8) {
    self.bg_palette[0] = SHADES[(value & 3) as usize];
    self.bg_palette[1] = SHADES[((value >> 2) & 3) as usize];
    self.bg_palette[2] = SHADES[((value >> 4) & 3) as usize];
    self.bg_palette[3] = SHADES[((value >> 6) & 3) as usize];
    self.bg_palette_value = value;
  }

  pub fn get_bgp(&self) -> u8 {
    self.bg_palette_value
  }

  pub fn set_obj_palette(&mut self, palette: usize, value: u8) {
    let offset = (palette & 7) * 4;
    self.object_palettes[offset + 0] = SHADES[(value & 3) as usize];
    self.object_palettes[offset + 1] = SHADES[((value >> 2) & 3) as usize];
    self.object_palettes[offset + 2] = SHADES[((value >> 4) & 3) as usize];
    self.object_palettes[offset + 3] = SHADES[((value >> 6) & 3) as usize];
    self.object_palette_values[palette & 7] = value;
  }

  pub fn get_obj_palette(&self, palette: usize) -> u8 {
    self.object_palette_values[palette & 7]
  }

  pub fn get_tile_address(&self, index: usize) -> usize {
    ((self.first_tile_offset + (index * 16)) & 0xfff)
      + self.tile_address_offset
  }

  fn get_bg_tile(&self, x: usize, y: usize, vram: &Box<[u8]>) -> u8 {
    let offset = x + y * 32;
    let address = self.bg_map_offset + offset;
    vram[address]
  }

  fn get_window_tile(&self, x: usize, y: usize, vram: &Box<[u8]>) -> u8 {
    let offset = x + y * 32;
    let address = self.window_map_offset + offset;
    vram[address]
  }

  pub fn get_tile_row(&self, video_ram: &Box<[u8]>, tile: usize, row: usize) -> u16 {
    let mut address = self.get_tile_address(tile);
    address += row * 2;
    let low = video_ram[address];
    let high = video_ram[address + 1];
    tile::interleave(low, high)
  }

  pub fn get_object_row(&self, video_ram: &Box<[u8]>, tile_index: usize, row: usize, flip_x: bool) -> u16 {
    let mut address = tile_index << 4;
    address += row * 2;
    let low_raw = video_ram[address];
    let high_raw = video_ram[address + 1];

    let low = if flip_x {
      (((low_raw as u64 * 0x80200802) & 0x0884422110).wrapping_mul(0x0101010101) >> 32) as u8
    } else {
      low_raw
    };
    let high = if flip_x {
      (((high_raw as u64 * 0x80200802) & 0x0884422110).wrapping_mul(0x0101010101) >> 32) as u8
    } else {
      high_raw
    };
    tile::interleave(low, high)
  }

  fn find_current_line_sprites(&mut self, video_ram: &Box<[u8]>, oam: &Box<[u8]>) {
    // clear the object line cache to start with a clean slate
    for i in 0..self.object_line_cache.len() {
      self.object_line_cache[i] = 0;
    }
    self.current_obj_line_cache_pixel = 8;
    
    if !self.object_enabled {
      return;
    }
    let current_line = self.current_line as isize;
    let object_height = if self.object_double_height { 16 } else { 8 };
    // iterate over all objects in OAM (every 4 bytes)
    let mut objects_found: Vec<Option<ObjectAttributes>> = Vec::with_capacity(10);
    let mut offset = 0;
    while offset < 160 && objects_found.len() < 10 {
      let object_y = oam[offset] as isize;
      let object_x = oam[offset + 1];
      let tile_index = oam[offset + 2] as usize;
      let attributes = oam[offset + 3];
      offset += 4;
      let mut object_line = current_line + 16 - object_y;
      if object_line < 0 || object_line >= object_height {
        continue;
      }
      let flip_y = attributes & 0x40 != 0;
      let flip_x = attributes & 0x20 != 0;
      if flip_y {
        object_line = object_height - object_line - 1;
      }

      let row_data = self.get_object_row(video_ram, tile_index, object_line as usize, flip_x);

      objects_found.push(
        Some(
          ObjectAttributes {
            has_priority: attributes & 0x80 != 0,
            palette: (attributes & 0x10) >> 4,
            row_data,
            x_coord: object_x,
          }
        )
      );
    }

    let total_objects = objects_found.len();
    if total_objects == 0 {
      return;
    }
    let mut drawn_count = 0;
    // line_x sweeps across every pixel in the object line cache
    // The first 8 pixels won't be drawn to the screen, nor the pixels beyond
    // index 168.
    let mut line_x = 0;
    // At each pixel, seek through the objects on the current line until one is
    // found starting on the current pixel. If one is located, its pixels are
    // copied to the unpopulated pixels in the line cache. After that, the
    // object is removed from the current set, so that future sweeps through
    // the object list can ignore it.
    while line_x < 168 && drawn_count < total_objects {
      for obj_index in 0..total_objects {
        let obj_found = objects_found.get(obj_index);

        let drawn = if let Some(Some(obj)) = obj_found {
          if obj.x_coord != line_x {
            false
          } else {
            // this object begins drawing at line_x
            // Copy enough data to object_line_cache to be rendered when the
            // LCD line is actually drawn.
            let mut pixel_data = obj.row_data;
            for x in 0..8 {
              let offset = (line_x as usize) + x;
              if self.object_line_cache[offset] & 0x80 == 0 {
                let priority = if obj.has_priority { 0x40 } else { 0 };
                let palette = obj.palette << 2;
                let color_index = ((pixel_data >> 14) & 3) as u8;
                if color_index != 0 {
                  self.object_line_cache[offset] = (
                    0x80 | // present
                    priority |
                    palette |
                    color_index
                  );
                }
              }
              pixel_data <<= 2;
            }
            drawn_count += 1;
            true
          }
        } else {
          false
        };
        if drawn {
          // remove the object from the set
          if let Some(inner) = objects_found.get_mut(obj_index) {
            let _ = inner.take();
          }
        }
      }

      line_x += 1;
    }
  }

  fn cache_next_tile_row(&mut self, vram: &Box<[u8]>) {
    let tile_x = self.next_cached_tile_x;
    let relative_tile_line = self.current_line.wrapping_add(self.scroll_y) as usize;
    let tile_y = relative_tile_line >> 3;
    let tile_index = self.get_bg_tile(tile_x, tile_y, vram) as usize;
    let tile_row = relative_tile_line & 7;
    self.current_tile_cache = self.get_tile_row(vram, tile_index, tile_row);
    self.next_cached_tile_x += 1;
    self.next_cached_tile_x %= 32;
  }

  fn cache_next_window_tile_row(&mut self, vram: &Box<[u8]>) {
    let tile_x = self.next_cached_tile_x;
    let relative_tile_line = self.current_line.wrapping_sub(self.window_y) as usize;
    let tile_y = relative_tile_line >> 3;
    let tile_index = self.get_window_tile(tile_x, tile_y, vram) as usize;
    let tile_row = relative_tile_line & 7;
    self.current_tile_cache = self.get_tile_row(vram, tile_index, tile_row);
    self.next_cached_tile_x += 1;
    self.next_cached_tile_x %= 32;
  }

  fn check_current_line(&self) -> InterruptFlag {
    if self.ly_compare == self.current_line {
      if self.interrupt_on_lyc {
        return InterruptFlag::stat();
      }
    }
    InterruptFlag::empty()
  }

  fn check_mode_interrupt(&self) -> InterruptFlag {
    if self.interrupt_on_mode_2 && self.current_mode == 2 {
      InterruptFlag::stat()
    } else if self.interrupt_on_mode_1 && self.current_mode == 1 {
      InterruptFlag::stat()
    } else if self.interrupt_on_mode_0 && self.current_mode == 0 {
      InterruptFlag::stat()
    } else {
      InterruptFlag::empty()
    }
  }

  pub fn run_clock_cycles(&mut self, cycles: ClockCycles, vram: &Box<[u8]>, oam: &Box<[u8]>) -> InterruptFlag {
    let mut cycles_remaining = cycles.as_usize();
    let mut interrupt_state = InterruptFlag::empty();
    while cycles_remaining > 0 {
      cycles_remaining -= 4;
      let previous_dot_count = self.current_mode_dots;
      self.current_mode_dots += 4;
      match self.current_mode {
        0 => {
          // Mode 3 takes a variable amount of time to draw the line, depending
          // on how many sprites are on the line.
          // Mode 3 and Mode 0 together take 376 dots.
          // While innacurate, this just splits the difference and assigns 188
          // dots to each of the two modes.
          // Also, 188 is perfectly divisible by 4, so the math works out
          // cleanly when drawing 4 dots at a time, for each machine cycle.
          if self.current_mode_dots >= 188 {
            self.current_mode_dots -= 188;
            if self.current_line < 144 {
              self.current_line += 1;
              self.current_mode = 2;
              interrupt_state |= self.check_mode_interrupt();
              interrupt_state |= self.check_current_line();
              // pre-compute up to 10 sprites that overlap the current line
              self.find_current_line_sprites(vram, oam);
            } else {
              // On line 144, enter VBLANK and set appropriate flags
              self.current_mode = 1;
              self.lcd.swap_buffers();
              interrupt_state |= self.check_mode_interrupt();
              interrupt_state |= InterruptFlag::vblank();
            }
          }
        },
        1 => {
          // Mode 1 is the VBLANK, and runs for 10 full invisible scanlines
          // Each scanline is 456 dots long.
          if self.current_mode_dots >= 456 {
            self.current_mode_dots -= 456;

            if self.current_line < 153 {
              self.current_line += 1;
              interrupt_state |= self.check_current_line();
            } else {
              // VBLANK ended, start in mode 2 on line 0
              self.current_line = 0;
              self.current_mode = 2;
              // pre-compute up to 10 sprites that overlap the current line
              self.find_current_line_sprites(vram, oam);
              interrupt_state |= self.check_mode_interrupt();
            }
          }
        },
        2 => {
          // During mode 2, the GB is searching for active sprites.
          // Since this is pre-computed on the first dot of mode 2, there's no
          // work to do here.
          if self.current_mode_dots >= 80 {
            self.current_mode_dots -= 80;
            self.current_mode = 3;

            // For each screen line, determine if part of the window is visible
            let mut use_window = false;
            self.current_window_line = if !self.window_enabled || self.current_line < self.window_y {
              None
            } else {
              use_window = true;
              Some((self.current_line - self.window_y) as usize)
            };
            if use_window && self.window_x <= 7 {
              // first tile drawn will be the window
              let first_window_pixel = 7 - self.window_x;
              self.next_cached_tile_x = 0;
              self.cache_next_window_tile_row(vram);
              let window_shift = first_window_pixel as usize * 2;
              self.current_tile_cache <<= window_shift;
            } else {
              // first tile drawn will be the bg
              self.next_cached_tile_x = (self.scroll_x >> 3) as usize % 32;
              self.cache_next_tile_row(vram);
              let fine_scroll_x = self.scroll_x as usize & 7;
              let shift = fine_scroll_x * 2;
              self.current_tile_cache <<= shift;
            }
          }
        },
        3 => {
          // During mode 3, the actual screen line is drawn.
          // As the dot counter is incremented, draw 4 dots to the line buffer
          // at a time. Each time the end of the current tile is reached,

          if self.current_mode_dots >= 188 {
            self.current_mode_dots -= 188;
            self.current_mode = 0;
            interrupt_state |= self.check_mode_interrupt();
          } else if self.current_mode_dots <= 160 && self.current_line < 144 {
            let mut tile_x: usize = previous_dot_count & 7;
            let fine_scroll_x = self.scroll_x as usize & 7;
            tile_x += fine_scroll_x;
            tile_x &= 7;

            let mut draw_window = false;
            let window_x = self.window_x as usize;
            if let Some(_y) = self.current_window_line {
              draw_window = true;
              if previous_dot_count + 7 >= window_x {
                // draw the window instead of the bg
                tile_x = (previous_dot_count + 7 - window_x) & 7;
              }
              // If this is the first pixel of the line, the tile will have
              // already been cached during Mode 2.
              // In other cases, it will have been cached once the draw loop
              // increments the current pixel and does a window check.
            }

            let mut dots_remaining: usize = 4;
            let mut current_write_index: usize = previous_dot_count;

            // Shift 4 pixels out of the current tile and into the line buffer.
            // If the end of the tile is reached, compute and cache the next tile.
            loop {
              let current_line_buffer = self.lcd.get_writing_buffer_line(self.current_line as usize);
              while tile_x < 8 && dots_remaining > 0 {
                // fetch a pixel out of the object line cache
                let object_pixel = self.object_line_cache[self.current_obj_line_cache_pixel];
                self.current_obj_line_cache_pixel += 1;

                // shift a pixel out of the current tile cache
                let palette_index = ((self.current_tile_cache & 0xc000) >> 14) as u8;
                let bg_color = self.bg_palette[palette_index as usize];

                // TODO: sort out sprite priority with bg
                if object_pixel & 0x80 != 0 && object_pixel & 3 != 0 {
                  // sprite is present
                  let palette_index = (object_pixel & 0x1c) >> 2;
                  let pal_offset = palette_index as usize * 4;
                  let obj_color = self.object_palettes[pal_offset + ((object_pixel & 3) as usize)];
                  current_line_buffer[current_write_index] = obj_color;
                } else {
                  current_line_buffer[current_write_index] = bg_color;
                }

                self.current_tile_cache <<= 2;
                tile_x += 1;
                dots_remaining -= 1;
                current_write_index += 1;

                if draw_window && current_write_index + 7 == window_x {
                  // the next pixel needs to be the start of the window
                  self.next_cached_tile_x = 0;
                  tile_x = 8;
                }
              }
              // At the end of the loop, either the tile has finished drawing
              // or time has expired and the video processor needs to yield to
              // other devices.
              if tile_x >= 8 {
                if draw_window && (current_write_index + 7 >= window_x) {
                  self.cache_next_window_tile_row(vram);
                } else {
                  self.cache_next_tile_row(vram);
                }
                tile_x = 0;
              }
              if dots_remaining == 0 {
                break;
              }
            }
          }
        },
        _ => unsafe { std::hint::unreachable_unchecked() },
      };
    }

    interrupt_state
  }

  pub fn get_writing_buffer(&self) -> &Box<[u8]> {
    self.lcd.get_writing_buffer_readonly()
  }

  pub fn get_visible_buffer(&self) -> &Box<[u8]> {
    self.lcd.get_visible_buffer()
  }
}

#[cfg(test)]
mod tests {
  use crate::timing::ClockCycles;
  use super::VideoState;

  #[test]
  fn tile_blocks() {
    let mut video = VideoState::new();
    assert_eq!(video.get_tile_address(0), 0);
    assert_eq!(video.get_tile_address(128), 0x800);
    assert_eq!(video.get_tile_address(255), 0xff0);
    video.set_lcd_control(0x80);
    assert_eq!(video.get_tile_address(0), 0x1000);
    assert_eq!(video.get_tile_address(128), 0x800);
    assert_eq!(video.get_tile_address(255), 0xff0);
  }

  #[test]
  fn video_mode_timing() {
    let mut vram_vec = Vec::with_capacity(0x2000);
    for _ in 0..0x2000 {
      vram_vec.push(0);
    }
    let mut oam_vec = Vec::with_capacity(0xa0);
    for _ in 0..0xa0 {
      oam_vec.push(0);
    }
    let mut vram = vram_vec.into_boxed_slice();
    let mut oam = oam_vec.into_boxed_slice();
    let mut video = VideoState::new();
    // video state starts out in vblank
    assert_eq!(video.get_lcd_status() & 3, 1);
    assert_eq!(video.get_ly(), 144);
    for i in 1..10 {
      video.run_clock_cycles(ClockCycles(456), &mut vram, &oam);
      assert_eq!(video.get_ly(), 144 + i);
    }
    video.run_clock_cycles(ClockCycles(456), &mut vram, &oam);
    // should now be in mode 2 of the first line
    assert_eq!(video.get_lcd_status() & 3, 2);
    assert_eq!(video.get_ly(), 0);
    video.run_clock_cycles(ClockCycles(80), &mut vram, &oam);
    // now in mode 3
    assert_eq!(video.get_lcd_status() & 3, 3);
    assert_eq!(video.get_ly(), 0);
    video.run_clock_cycles(ClockCycles(376), &mut vram, &oam);
    // 376 cycles covers mode 3 and mode 0, skipping to the next line
    assert_eq!(video.get_lcd_status() & 3, 2);
    assert_eq!(video.get_ly(), 1);
  }

  #[test]
  fn basic_bg_drawing() {
    let mut vram_vec = Vec::with_capacity(0x2000);
    for _ in 0..0x2000 {
      vram_vec.push(0);
    }
    let mut oam_vec = Vec::with_capacity(0xa0);
    for _ in 0..0xa0 {
      oam_vec.push(0);
    }
    let mut vram = vram_vec.into_boxed_slice();
    let mut oam = oam_vec.into_boxed_slice();
    {
      // make the first line of the bg a repeating pattern of the first 4 tiles
      for i in 0..32 {
        vram[0x1800 + i] = (i & 3) as u8;
      }

      // make the first 4 tiles different alternating lines of gray
      for y in 0..8 {
        vram[y * 2] = if y & 1 == 0 { 0 } else { 0xff };
        vram[y * 2 + 1] = if y & 1 == 0 { 0 } else { 0xff };
      }
      for y in 0..8 {
        vram[16 + y * 2] = if y & 1 == 0 { 0xff } else { 0 };
        vram[16 + y * 2 + 1] = if y & 1 == 0 { 0 } else { 0xff };
      }
      for y in 0..8 {
        vram[32 + y * 2] = if y & 1 == 0 { 0 } else { 0xff };
        vram[32 + y * 2 + 1] = if y & 1 == 0 { 0xff } else { 0 };
      }
      for y in 0..8 {
        vram[48 + y * 2] = if y & 1 == 0 { 0xff } else { 0 };
        vram[48 + y * 2 + 1] = if y & 1 == 0 { 0xff } else { 0 };
      }
    }
    let mut video = VideoState::new();
    video.set_bgp(0b11100100);
    video.set_lcd_control(0x90); // enable LCD, tiles start at 0x8000
    // get to start of first line
    video.run_clock_cycles(ClockCycles(456 * 10), &mut vram, &oam);
    // draw first line
    video.run_clock_cycles(ClockCycles(456), &mut vram, &oam);
    for i in 0..8 {
      assert_eq!(video.get_writing_buffer()[i], 255); // white
    }
    for i in 8..16 {
      assert_eq!(video.get_writing_buffer()[i], 170); // light gray
    }
    for i in 16..24 {
      assert_eq!(video.get_writing_buffer()[i], 85); // dark gray
    }
    for i in 24..32 {
      assert_eq!(video.get_writing_buffer()[i], 0); // black
    }
    for i in 32..40 {
      assert_eq!(video.get_writing_buffer()[i], 255);
    }
    for i in 40..48 {
      assert_eq!(video.get_writing_buffer()[i], 170);
    }
    for i in 48..56 {
      assert_eq!(video.get_writing_buffer()[i], 85);
    }
    for i in 56..64 {
      assert_eq!(video.get_writing_buffer()[i], 0);
    }

    // draw second line
    video.run_clock_cycles(ClockCycles(456), &mut vram, &oam);
    for i in 0..8 {
      assert_eq!(video.get_writing_buffer()[160 + i], 0);
    }
    for i in 8..16 {
      assert_eq!(video.get_writing_buffer()[160 + i], 85);
    }
    for i in 16..24 {
      assert_eq!(video.get_writing_buffer()[160 + i], 170);
    }
    for i in 24..32 {
      assert_eq!(video.get_writing_buffer()[160 + i], 255);
    }
  }
}

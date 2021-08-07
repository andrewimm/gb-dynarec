pub mod lcd;
pub mod tile;

use lcd::LCD;
use super::interrupts::InterruptFlag;

pub struct VideoState {
  lcd: LCD,
  tile_address_offset: usize,
  first_tile_offset: usize,
  lcd_control_value: u8,
  ly_compare: u8,

  current_mode: u8,
  current_mode_dots: usize,
  current_line: u8,
}

impl VideoState {
  pub fn new() -> Self {
    Self {
      lcd: LCD::new(),
      tile_address_offset: 0,
      first_tile_offset: 0,
      lcd_control_value: 0,
      ly_compare: 0,

      // start at the beginning of a vblank
      current_mode: 1,
      current_mode_dots: 0,
      current_line: 144,
    }
  }

  pub fn set_lcd_control(&mut self, value: u8) {
    self.lcd.set_enabled(value & 0x80 != 0);

    let (tile_address_offset, first_tile_offset) = if value & 0x10 == 0 {
      (0x800, 0x800)
    } else {
      (0, 0)
    };
    self.tile_address_offset = tile_address_offset;
    self.first_tile_offset = first_tile_offset;
    self.lcd_control_value = value;
  }

  pub fn get_lcd_control(&self) -> u8 {
    self.lcd_control_value
  }

  pub fn set_lcd_status(&mut self, value: u8) {

  }

  pub fn get_lcd_status(&self) -> u8 {
    let mut status = 0;

    if self.ly_compare == self.current_line {
      status |= 4;
    }
    // set the lower two bits to the current mode
    status |= self.current_mode;
    status
  }

  pub fn set_scroll_x(&mut self, value: u8) {

  }

  pub fn get_scroll_x(&self) -> u8 {
    0
  }

  pub fn set_scroll_y(&mut self, value: u8) {

  }

  pub fn get_scroll_y(&self) -> u8 {
    0
  }

  pub fn get_ly(&self) -> u8 {
    self.current_line
  }

  pub fn set_ly_compare(&mut self, value: u8) {
    self.ly_compare = value;
  }

  pub fn get_ly_compare(&self) -> u8 {
    self.ly_compare
  }

  pub fn get_tile_address(&self, index: usize) -> usize {
    ((self.first_tile_offset + (index * 16)) & 0xfff)
      + self.tile_address_offset
  }

  pub fn get_tile_row(&self, video_ram: &Box<[u8]>, tile: usize, row: usize) -> u16 {
    let mut address = self.get_tile_address(tile);
    address += row * 2;
    let low = video_ram[address];
    let high = video_ram[address + 1];
    tile::interleave(low, high)
  }

  fn find_current_line_sprites(&mut self) {

  }

  pub fn run_clock_cycles(&mut self, cycles: usize, vram: &Box<[u8]>) -> InterruptFlag {
    let mut cycles_remaining = cycles;
    while cycles_remaining > 0 {
      cycles_remaining -= 1;
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
              // pre-compute up to 10 sprites that overlap the current line
              self.find_current_line_sprites();
            } else {
              // On line 144, enter VBLANK and set appropriate flags
              self.current_mode = 1;
              return InterruptFlag::vblank();
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
            } else {
              // VBLANK ended, start in mode 2 on line 0
              self.current_line = 0;
              self.current_mode = 2;
              // pre-compute up to 10 sprites that overlap the current line
              self.find_current_line_sprites();
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
          }
        },
        3 => {
          // During mode 3, the actual screen line is drawn.
          // As the dot counter is incremented, draw 4 dots to the line buffer
          // at a time. Each time the end of the current tile is reached,

          if self.current_mode_dots >= 188 {
            self.current_mode_dots -= 188;
            self.current_mode = 0;
          }

          // TODO: Account for scroll-x
          let mut tile_x: usize = previous_dot_count & 7;
          let mut dots_remaining: usize = 4;
          // Shift 4 pixels out of the current tile and into the line buffer.
          // If the end of the tile is reached, compute and cache the next tile.
          loop {
            while tile_x < 8 && dots_remaining > 0 {
              // shift a pixel out of the current tile cache

              tile_x += 1;
              dots_remaining -= 1;
            }
            if dots_remaining > 0 {
              // load the next tile to cache

            } else {
              break;
            }
          }
        },
        _ => unsafe { std::hint::unreachable_unchecked() },
      };
    }

    InterruptFlag::empty()
  }
}

#[cfg(test)]
mod tests {
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
    let mut vram = Vec::new().into_boxed_slice();
    let mut video = VideoState::new();
    // video state starts out in vblank
    assert_eq!(video.get_lcd_status() & 3, 1);
    assert_eq!(video.get_ly(), 144);
    for i in 1..10 {
      video.run_clock_cycles(456 / 4, &mut vram);
      assert_eq!(video.get_ly(), 144 + i);
    }
    video.run_clock_cycles(456 / 4, &mut vram);
    // should now be in mode 2 of the first line
    assert_eq!(video.get_lcd_status() & 3, 2);
    assert_eq!(video.get_ly(), 0);
    video.run_clock_cycles(80 / 4, &mut vram);
    // now in mode 3
    assert_eq!(video.get_lcd_status() & 3, 3);
    assert_eq!(video.get_ly(), 0);
    video.run_clock_cycles(376 / 4, &mut vram);
    // 376 cycles covers mode 3 and mode 0, skipping to the next line
    assert_eq!(video.get_lcd_status() & 3, 2);
    assert_eq!(video.get_ly(), 1);
  }
}
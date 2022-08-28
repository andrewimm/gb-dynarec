pub struct LCD {
  visible_buffer: Box<[u8]>,
  writing_buffer: Box<[u8]>,
  enabled: bool,
}

pub const LCD_WIDTH: usize = 160;
pub const LCD_HEIGHT: usize = 144;
const LCD_SIZE: usize = LCD_WIDTH * LCD_HEIGHT;

impl LCD {
  pub fn new() -> Self {
    let mut visible: Vec<u8> = Vec::with_capacity(LCD_SIZE);
    let mut writing: Vec<u8> = Vec::with_capacity(LCD_SIZE);
    for _ in 0..LCD_SIZE {
      visible.push(0);
      writing.push(0);
    }
    let visible_buffer = visible.into_boxed_slice();
    let writing_buffer = writing.into_boxed_slice();
    
    Self {
      visible_buffer,
      writing_buffer,
      enabled: true,
    }
  }

  pub fn get_visible_buffer(&self) -> &Box<[u8]> {
    &self.visible_buffer
  }

  pub fn get_writing_buffer(&mut self) -> &mut Box<[u8]> {
    &mut self.writing_buffer
  }

  pub fn get_writing_buffer_readonly(&self) -> &Box<[u8]> {
    &self.writing_buffer
  }

  pub fn get_writing_buffer_line(&mut self, line: usize) -> &mut [u8] {
    let start = line * LCD_WIDTH;
    let end = start + LCD_WIDTH;
    &mut self.writing_buffer[start..end]
  }

  pub fn swap_buffers(&mut self) {
    std::mem::swap(&mut self.visible_buffer, &mut self.writing_buffer);
  }

  pub fn set_enabled(&mut self, enabled: bool) {
    self.enabled = enabled;
  }

  pub fn is_enabled(&self) -> bool {
    self.enabled
  }
}

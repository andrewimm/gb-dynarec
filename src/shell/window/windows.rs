use crate::bindings::{
  Windows::Win32::Foundation::{HANDLE, HWND},
  Windows::Win32::Graphics::Gdi::{
    BITMAPINFO,
    BITMAPINFOHEADER,
    DIB_RGB_COLORS,
    HBITMAP,
    HDC,
    SRCCOPY,
    BitBlt,
    CreateCompatibleDC,
    CreateDIBSection,
    DeleteObject,
    GetDC,
    ReleaseDC,
    SelectObject,
  },
};
use crate::emulator::Core;
use super::super::Shell;
use std::time::Duration;
use winit::{
  dpi::PhysicalSize,
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  platform::windows::WindowExtWindows,
  window::WindowBuilder,
};

pub struct WindowShell {
  scale: usize,
}

impl WindowShell {
  pub fn new() -> Self {
    Self {
      scale: 4,
    }
  }

  pub fn get_width(&self) -> f64 {
    self.scale as f64 * 160.0
  }

  pub fn get_height(&self) -> f64 {
    self.scale as f64 * 144.0
  }
}

impl Shell for WindowShell {
  fn run(&mut self, mut core: Core) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
      .with_title("gb-dynarec")
      .with_inner_size(PhysicalSize::new(self.get_width(), self.get_height()))
      .build(&event_loop)
      .expect("Failed to initialize window");
    
    let scale = self.scale;

    let hwnd = HWND(window.hwnd() as isize);
    
    let (bitmap, bitmap_memory) = unsafe {
      use std::ffi::c_void;

      let mut info: BITMAPINFO = std::mem::zeroed();
      info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
      info.bmiHeader.biWidth = 160 * scale as i32;
      info.bmiHeader.biHeight = -144 * scale as i32;
      info.bmiHeader.biPlanes = 1;
      info.bmiHeader.biBitCount = 24;
      info.bmiHeader.biCompression = 0;

      let mut bits_ptr: *mut c_void = std::ptr::null_mut();

      let bitmap: HBITMAP = CreateDIBSection(
        HDC::NULL,
        &info as *const BITMAPINFO,
        DIB_RGB_COLORS,
        &mut bits_ptr as *mut *mut c_void,
        HANDLE::NULL,
        0,
      );

      let row_size = ((160 * scale * 24 + 31) / 32) * 4;
      let image_size = row_size * 144 * scale;
      let image_slice: &mut [u8] = std::slice::from_raw_parts_mut(bits_ptr as *mut u8, image_size);

      (bitmap, image_slice)
    };
    
    event_loop.run(move |event, _, control_flow| {
      match event {
        Event::WindowEvent {
          event: WindowEvent::CloseRequested,
          window_id,
        } => {
          if window_id == window.id() {
            // clean up?
            
            *control_flow = ControlFlow::Exit;
          }
        },
        Event::MainEventsCleared => {
          core.run_frame();

          let lcd_data = core.get_screen_buffer();
          copy_lcd(bitmap_memory, lcd_data, scale);
          unsafe {
            draw(hwnd, bitmap, scale);
          }
        },
        _ => *control_flow = ControlFlow::Poll,
      }
    });
  }
}

pub fn copy_lcd(bitmap_memory: &mut [u8], lcd_data: &Box<[u8]>, scale: usize) {
  let width = 160 * scale;
  let row_size = ((width * 24 + 31) / 32) * 4;

  for x in 0..(160 * scale) {
    for y in 0..(144 * scale) {
      let src_x = x / scale;
      let src_y = y / scale;
      let src_index = src_y * 160 + src_x;
      let pixel = lcd_data[src_index];
      let offset = y * row_size + x * 3;
      bitmap_memory[offset + 0] = pixel; // B
      bitmap_memory[offset + 1] = pixel; // R
      bitmap_memory[offset + 2] = pixel; // G
    }
  }
}

unsafe fn draw(hwnd: HWND, bitmap: HBITMAP, scale: usize) {
  let hdc = GetDC(hwnd);
  let dc_mem = CreateCompatibleDC(hdc);

  let old_bitmap = SelectObject(dc_mem, bitmap);

  BitBlt(hdc, 0, 0, 160 * scale as i32, 144 * scale as i32, dc_mem, 0, 0, SRCCOPY);

  SelectObject(dc_mem, old_bitmap);

  ReleaseDC(hwnd, hdc);
}

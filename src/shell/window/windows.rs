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
use winit::{
  dpi::LogicalSize,
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  platform::windows::WindowExtWindows,
  window::WindowBuilder,
};

pub struct WindowShell {
  scale: f64,
}

impl WindowShell {
  pub fn new() -> Self {
    Self {
      scale: 1.0,
    }
  }

  pub fn get_width(&self) -> f64 {
    self.scale * 160.0
  }

  pub fn get_height(&self) -> f64 {
    self.scale * 144.0
  }
}

impl Shell for WindowShell {
  fn run(&mut self, mut core: Core) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
      .with_title("gb-dynarec")
      .with_inner_size(LogicalSize::new(self.get_width(), self.get_height()))
      .build(&event_loop)
      .expect("Failed to initialize window");

    let hwnd = HWND(window.hwnd() as isize);
    
    let (bitmap, bitmap_memory) = unsafe {
      use std::ffi::c_void;

      let mut info: BITMAPINFO = std::mem::zeroed();
      info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
      info.bmiHeader.biWidth = 160;
      info.bmiHeader.biHeight = 144;
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

      let row_size = ((160 * 24 + 31) / 32) * 4;
      let image_size = row_size * 144;
      let image_slice: &mut [u8] = std::slice::from_raw_parts_mut(bits_ptr as *mut u8, image_size);

      for y in 0..144 {
        for x in 0..160 {
          let offset = y * row_size + x * 3;
          image_slice[offset + 0] = 255;
          image_slice[offset + 1] = 0;
          image_slice[offset + 2] = 0;
        }
      }

      (bitmap, image_slice)
    };
    
    event_loop.run(move |event, _, control_flow| {
      *control_flow = ControlFlow::Poll;

      match event {
        Event::WindowEvent {
          event: WindowEvent::CloseRequested,
          window_id,
        } => {
          if window_id == window.id() {
            // clean up
            unsafe {
              DeleteObject(bitmap);
            }
            
            *control_flow = ControlFlow::Exit;
          }
        },
        _ => {
          unsafe {
            draw(hwnd, bitmap);
          }
        },
      }
    });
  }
}

unsafe fn draw(hwnd: HWND, bitmap: HBITMAP) {
  let hdc = GetDC(hwnd);
  let dc_mem = CreateCompatibleDC(hdc);

  let old_bitmap = SelectObject(dc_mem, bitmap);

  BitBlt(hdc, 0, 0, 160, 144, dc_mem, 0, 0, SRCCOPY);

  SelectObject(dc_mem, old_bitmap);

  ReleaseDC(hwnd, hdc);
}

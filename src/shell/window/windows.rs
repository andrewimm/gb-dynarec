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
    GetDC,
    ReleaseDC,
    SelectObject,
  },
};
use raw_window_handle::Win32WindowHandle;
use super::VideoImpl;
use winit::{
  dpi::PhysicalSize,
};

pub struct Video {
  scale: usize,
  hwnd: HWND,
  bitmap: HBITMAP,
  bitmap_raw_ptr: *mut u8,
  bitmap_length: usize,
}

impl Video {
  pub fn new(window_handle: Win32WindowHandle) -> Self {
    let scale = super::INITIAL_SCALE;

    let row_size = ((160 * scale * 24 + 31) / 32) * 4;
    let image_size = row_size * 144 * scale;

    let hwnd = HWND(window_handle.hwnd as isize);
    let (bitmap, bitmap_raw_ptr) = unsafe {
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

      (bitmap, bits_ptr as *mut u8)
    };

    Self {
      scale,
      hwnd,
      bitmap,
      bitmap_raw_ptr,
      bitmap_length: image_size,
    }
  }
}

impl VideoImpl for Video {
  fn draw_lcd(&mut self, lcd_data: &[u8]) {
    let scale = self.scale;

    unsafe {
      let bitmap_memory: &mut [u8] = std::slice::from_raw_parts_mut(self.bitmap_raw_ptr, self.bitmap_length);
      // Copy LCD data to bitmap
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

      // draw bitmap to screen
      let hdc = GetDC(self.hwnd);
      let dc_mem = CreateCompatibleDC(hdc);
      let old_bitmap = SelectObject(dc_mem, self.bitmap);
      BitBlt(hdc, 0, 0, 160 * scale as i32, 144 * scale as i32, dc_mem, 0, 0, SRCCOPY);
      SelectObject(dc_mem, old_bitmap);
      ReleaseDC(self.hwnd, hdc);
    }
  }

  fn increase_scale(&mut self) -> PhysicalSize<u32> {
    if self.scale >= 8 {
      return PhysicalSize::new(160 * self.scale as u32, 144 * self.scale as u32);
    }
    let new_scale = self.scale * 2;
    //self.set_scale(new_scale);
    PhysicalSize::new(160 * new_scale as u32, 144 * new_scale as u32)
  }

  fn decrease_scale(&mut self) -> PhysicalSize<u32> {
    if self.scale <= 1 {
      return PhysicalSize::new(160 * self.scale as u32, 144 * self.scale as u32);
    }
    let new_scale = self.scale / 2;
    //self.set_scale(new_scale);
    PhysicalSize::new(160 * new_scale as u32, 144 * new_scale as u32)
  }
}

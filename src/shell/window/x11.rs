use crate::emulator::Core;
use raw_window_handle::{
  HasRawDisplayHandle,
  HasRawWindowHandle,
  RawDisplayHandle,
  RawWindowHandle,
};
use super::super::Shell;
use winit::{
  dpi::PhysicalSize,
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
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
      .with_title(super::WINDOW_TITLE)
      .with_inner_size(PhysicalSize::new(self.get_width(), self.get_height()))
      .build(&event_loop)
      .expect("Failed to initialize window");
    
    let scale = self.scale;

    let xlib = x11_dl::xlib::Xlib::open().expect("Failed to open xlib");
    let window_handle = match window.raw_window_handle() {
      RawWindowHandle::Xlib(raw_handle) => raw_handle,
      _ => { // eventually with wayland support this check can be moved higher
        panic!("Unsupported window type");
      },
    };
    let drawable = window_handle.window;
    let display_handle = match window.raw_display_handle() {
      RawDisplayHandle::Xlib(raw_handle) => raw_handle,
      _ => panic!("Unsupported display type"),
    };
    let display = display_handle.display as *mut x11_dl::xlib::Display;
    let (screen, visual, depth, graphics_context) = unsafe {
      let screen = (xlib.XDefaultScreen)(display);
      let visual = (xlib.XDefaultVisual)(display, screen);
      let depth = (xlib.XDefaultDepth)(display, screen) as u32;
      let graphics_context = (xlib.XDefaultGC)(display, screen);

      (screen, visual, depth, graphics_context)
    };

    let mut bitmap_data = {
      let size = (160 * scale) * (144 * scale) * 4;
      let mut data = Vec::<u8>::with_capacity(size);
      for _ in 0..size {
        data.push(0);
      }
      data.into_boxed_slice()
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
          // draw lcd data
          {
            let width = 160 * scale;
            let row_size = width * 4;

            for x in 0..(160 * scale) {
              for y in 0..(144 * scale) {
                let src_x = x / scale;
                let src_y = y / scale;
                let src_index = src_y * 160 + src_x;
                let pixel = lcd_data[src_index];
                let offset = y * row_size + x * 4;
                bitmap_data[offset + 0] = pixel; // R
                bitmap_data[offset + 1] = pixel; // G
                bitmap_data[offset + 2] = pixel; // B
                bitmap_data[offset + 3] = 255;
              }
            }
          }
          let width = (160 * scale) as u32;
          let height = (144 * scale) as u32;
          unsafe {
            let image = (xlib.XCreateImage)(
              display,
              visual,
              depth,
              x11_dl::xlib::ZPixmap, // format
              0, // offset
              bitmap_data.as_ptr() as *mut std::os::raw::c_char,
              width,
              height,
              32, // scanline padding
              width as i32 * 4, // bytes per line
            );

            (xlib.XPutImage)(
              display,
              drawable,
              graphics_context,
              image,
              0, // x offset
              0, // y offset
              0, // x dest
              0, // y dest
              width,
              height,
            );

            // XDestroyImage attempts to free the data pointer as well,
            // so we need to point to null instead
            (*image).data = std::ptr::null_mut();

            (xlib.XDestroyImage)(image);
          }
        },
        _ => *control_flow = ControlFlow::Poll,
      }
    });
  }
}

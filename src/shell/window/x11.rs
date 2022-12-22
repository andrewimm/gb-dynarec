use raw_window_handle::{XlibDisplayHandle, XlibWindowHandle};
use super::VideoImpl;
use winit::dpi::PhysicalSize;

pub struct Video {
  scale: usize,
  video_buffer: Box<[u8]>,

  xlib: x11_dl::xlib::Xlib,
  display: *mut x11_dl::xlib::Display,
  visual: *mut x11_dl::xlib::Visual,
  depth: u32,
  drawable: u64,
  graphics_context: x11_dl::xlib::GC,
}

fn buffer_for_scale(scale: usize) -> Box<[u8]> {
  let bytes_per_pixel = 4;
  let size = (160 * scale) * (144 * scale) * bytes_per_pixel;
  let mut buffer = Vec::<u8>::with_capacity(size);
  for _ in 0..size {
    buffer.push(0);
  }
  buffer.into_boxed_slice()
}

impl Video {
  pub fn new(window_handle: XlibWindowHandle, display_handle: XlibDisplayHandle) -> Self {
    let scale = super::INITIAL_SCALE;
    let video_buffer = buffer_for_scale(scale);

    unsafe {
      let xlib = x11_dl::xlib::Xlib::open().expect("Failed to open xlib");
      let display = display_handle.display as *mut x11_dl::xlib::Display;
      let screen = (xlib.XDefaultScreen)(display);
      let visual = (xlib.XDefaultVisual)(display, screen);
      let depth = (xlib.XDefaultDepth)(display, screen) as u32;
      let graphics_context = (xlib.XDefaultGC)(display, screen);

      Self {
        scale,
        video_buffer,

        xlib,
        display,
        visual,
        depth,
        drawable: window_handle.window,
        graphics_context,
      }
    }
  }

  pub fn get_width(&self) -> f64 {
    self.scale as f64 * 160.0
  }

  pub fn get_height(&self) -> f64 {
    self.scale as f64 * 144.0
  }

  pub fn set_scale(&mut self, scale: usize) {
    if self.scale != scale {
      unsafe {
        //(self.xlib.XClearWindow)(self.display, self.drawable);
        (self.xlib.XFillRectangle)(
          self.display,
          self.drawable,
          self.graphics_context,
          0, 0,
          160 * self.scale as u32,
          144 * self.scale as u32,
        );
      }
    }
    self.scale = scale;
    let new_buffer = buffer_for_scale(scale);
    let old_buffer = std::mem::replace(&mut self.video_buffer, new_buffer);
    std::mem::forget(old_buffer);
  }
}

impl VideoImpl for Video {
  fn draw_lcd(&mut self, lcd_data: &[u8]) {
    // convert lcd_data into RGBA values for the local image buffer
    let scale = self.scale;
    let width = 160 * scale;
    let height = 144 * scale;
    let row_size = width * 4;

    let bitmap_data = &mut self.video_buffer;

    for x in 0..width {
      for y in 0..height {
        let src_x = x / scale;
        let src_y = y / scale;
        let src_index = src_y * 160 + src_x;
        let pixel = lcd_data[src_index];
        let offset = y * row_size + x * 4;
        bitmap_data[offset + 0] = pixel; // R
        bitmap_data[offset + 1] = pixel; // G
        bitmap_data[offset + 2] = pixel; // B
        bitmap_data[offset + 3] = 255;   // A
      }
    }

    unsafe {
      let image = (self.xlib.XCreateImage)(
        self.display,
        self.visual,
        self.depth,
        x11_dl::xlib::ZPixmap, // format
        0, // offset
        self.video_buffer.as_ptr() as *mut std::os::raw::c_char,
        width as u32,
        height as u32,
        32, // scanline padding
        width as i32 * 4, // bytes per line
      );

      (self.xlib.XPutImage)(
        self.display,
        self.drawable,
        self.graphics_context,
        image,
        0, // x offset
        0, // y offset
        0, // x dest
        0, // y dest
        width as u32,
        height as u32,
      );

      // XDestroyImage attempts to free the data pointer as well,
      // so we need to point to null instead
      (*image).data = std::ptr::null_mut();

      (self.xlib.XDestroyImage)(image);
    }
  }

  fn increase_scale(&mut self) -> PhysicalSize<u32> {
    if self.scale >= 8 {
      return PhysicalSize::new(160 * self.scale as u32, 144 * self.scale as u32);
    }
    let new_scale = self.scale * 2;
    self.set_scale(new_scale);
    PhysicalSize::new(160 * new_scale as u32, 144 * new_scale as u32)
  }

  fn decrease_scale(&mut self) -> PhysicalSize<u32> {
    if self.scale <= 1 {
      return PhysicalSize::new(160 * self.scale as u32, 144 * self.scale as u32);
    }
    let new_scale = self.scale / 2;
    self.set_scale(new_scale);
    PhysicalSize::new(160 * new_scale as u32, 144 * new_scale as u32)
  }
}

/*
impl Shell for WindowShell {
  fn run(&mut self, mut core: Core) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
      .with_title(super::WINDOW_TITLE)
      .with_inner_size(PhysicalSize::new(self.get_width(), self.get_height()))
      .build(&event_loop)
      .expect("Failed to initialize window");
    
    let scale = self.scale;

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

            let bitmap_data = &mut self.video_buffer;

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
              self.video_buffer.as_ptr() as *mut std::os::raw::c_char,
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
*/


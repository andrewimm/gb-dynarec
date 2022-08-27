use crate::emulator::Core;
use raw_window_handle::{
  HasRawDisplayHandle,
  HasRawWindowHandle,
  RawDisplayHandle,
  RawWindowHandle,
};
use winit::{
  dpi::PhysicalSize,
  event::{ElementState, Event, VirtualKeyCode, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder,
};

#[cfg(windows)]
pub mod windows;
#[cfg(unix)]
pub mod x11;

pub static WINDOW_TITLE: &str = "GB DYNAREC";
pub const INITIAL_SCALE: usize = 4;

pub struct WindowShell {}

impl WindowShell {
  pub fn new() -> Self {
    Self{}
  }
}

impl super::Shell for WindowShell {
  fn run(&mut self, mut core: Core) {
    let initial_width = 160 * INITIAL_SCALE as u32;
    let initial_height = 144 * INITIAL_SCALE as u32;
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
      .with_title(WINDOW_TITLE)
      .with_inner_size(PhysicalSize::new(initial_width, initial_height))
      .build(&event_loop)
      .expect("Failed to initialize window");

    let mut video_impl: Box<dyn VideoImpl> = match window.raw_window_handle() {
      #[cfg(windows)]
      RawWindowHandle::Win32(handle) => {
        Box::new(windows::Video::new(handle))
      },
      #[cfg(unix)]
      RawWindowHandle::Xlib(window_handle) => {
        let display_handle = match window.raw_display_handle() {
          RawDisplayHandle::Xlib(raw_handle) => raw_handle,
          _ => panic!("Display type does not match window type"),
        };
        Box::new(x11::Video::new(window_handle, display_handle))
      },
      #[cfg(unix)]
      RawWindowHandle::Wayland(handle) => {
        panic!("Wayland is not supported yet");
      },
      _ => panic!("Unsupported platform"),
    };

    event_loop.run(move |event, _, control_flow| {
      match event {
        Event::WindowEvent {
          event: e,
          window_id,
        } => {
          if window_id == window.id() {
            match e {
              WindowEvent::CloseRequested => {
                // clean up?

                *control_flow = ControlFlow::Exit;
              },
              WindowEvent::KeyboardInput { input, .. } => {
                let pressed = input.state == ElementState::Pressed;
                let is_ctrl = input.modifiers.ctrl();
                match input.virtual_keycode {
                  Some(VirtualKeyCode::Equals) => {
                    if is_ctrl && pressed {
                      let new_size = video_impl.increase_scale();
                      window.set_inner_size(new_size);
                    }
                  },
                  Some(VirtualKeyCode::Minus) => {
                    if is_ctrl && pressed {
                      let new_size = video_impl.decrease_scale();
                      window.set_inner_size(new_size);
                    }
                  },
                  _ => (),
                }
                *control_flow = ControlFlow::Poll;
              },
              _ => {
                *control_flow = ControlFlow::Poll;
              },
            }
          }
        },
        Event::MainEventsCleared => {
          core.run_frame();

          // get latest lcd data
          let lcd_data = core.get_screen_buffer();
          // draw lcd data to screen
          video_impl.draw_lcd(lcd_data);
        },
        _ => *control_flow = ControlFlow::Poll,
      }
    });

  }
}

pub trait VideoImpl {
  fn draw_lcd(&mut self, lcd_data: &[u8]);
  fn increase_scale(&mut self) -> PhysicalSize<u32>;
  fn decrease_scale(&mut self) -> PhysicalSize<u32>;
}

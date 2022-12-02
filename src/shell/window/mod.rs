use crate::emulator::Core;
use crate::devices::joypad::Button;
use raw_window_handle::{
  HasRawDisplayHandle,
  HasRawWindowHandle,
  RawDisplayHandle,
  RawWindowHandle,
};
use std::time::{Duration, SystemTime};
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

    let mut last_frame_time = SystemTime::now();

    event_loop.run(move |event, _, control_flow| {
      *control_flow = ControlFlow::Poll;

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
                  Some(code) => {
                    if let KeyboardInput::Joypad(b) = KeyboardInput::from_raw_input(code) {
                      if pressed {
                        core.memory.io.joypad.press_button(b);
                      } else {
                        core.memory.io.joypad.release_button(b);
                      }
                    }
                  },
                  _ => (),
                }
              },
              _ => (),
            }
          }
        },
        Event::MainEventsCleared => {
          let now = SystemTime::now();
          let mut elapsed = match now.duration_since(last_frame_time) {
            Ok(n) => n.as_millis(),
            Err(_) => 1,
          };
          last_frame_time = now;

          if elapsed < 16 {
            let diff = 16 - elapsed;
            let sleep_time = Duration::from_millis(diff as u64);
            std::thread::sleep(sleep_time);
            elapsed += diff;
          }

          core.run_frame();

          // get latest lcd data
          let lcd_data = core.get_screen_buffer();
          // draw lcd data to screen
          video_impl.draw_lcd(lcd_data);
        },
        _ => (),
      }
    });

  }
}

pub trait VideoImpl {
  fn draw_lcd(&mut self, lcd_data: &[u8]);
  fn increase_scale(&mut self) -> PhysicalSize<u32>;
  fn decrease_scale(&mut self) -> PhysicalSize<u32>;
}

pub enum KeyboardInput {
  Joypad(Button),
  Unknown,
}

impl KeyboardInput {
  pub fn from_raw_input(code: VirtualKeyCode) -> Self {
    match code {
      VirtualKeyCode::Z => KeyboardInput::Joypad(Button::B),
      VirtualKeyCode::X => KeyboardInput::Joypad(Button::A),
      VirtualKeyCode::Return => KeyboardInput::Joypad(Button::Start),
      
      VirtualKeyCode::Up => KeyboardInput::Joypad(Button::Up),
      VirtualKeyCode::Left => KeyboardInput::Joypad(Button::Left),
      VirtualKeyCode::Right => KeyboardInput::Joypad(Button::Right),
      VirtualKeyCode::Down => KeyboardInput::Joypad(Button::Down),

      _ => KeyboardInput::Unknown,
    }
  }
}

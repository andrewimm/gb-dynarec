#[cfg(not(feature="graphics"))]
mod headless;
#[cfg(feature="graphics")]
mod window;

#[cfg(not(feature="graphics"))]
use headless::HeadlessShell as ShellImpl;
#[cfg(feature="graphics")]
use window::WindowShell as ShellImpl;

use crate::emulator::Core;

pub trait Shell {
  fn run(&mut self, core: Core);
}

pub fn create_shell() -> ShellImpl {
  ShellImpl::new()
}

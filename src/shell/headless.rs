use crate::emulator::{self, Core};
use super::Shell;

pub struct HeadlessShell {

}

impl HeadlessShell {
  pub fn new() -> Self {
    Self {
      
    }
  }
}

impl Shell for HeadlessShell {
  fn run(&mut self, mut core: Core) {
    loop {
      core.update();
    }
  }
}

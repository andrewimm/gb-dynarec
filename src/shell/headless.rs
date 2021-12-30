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
      /*
      match core.run_state {
        emulator::RunState::Run => core.run_code_block(),
        emulator::RunState::Halt => {
          // don't run CPU until an interrupt
        },
        emulator::RunState::Stop => {
          // display is disabled until an interrupt
        },
      }
      */
    }
  }
}

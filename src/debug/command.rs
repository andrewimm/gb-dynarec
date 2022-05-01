//! Command prompt for the interactive debugger

use std::str::FromStr;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Command {
  /// Clear a breakpoint at a specific address
  BreakClear(u16),
  /// Return all active breakpoints
  BreakList,
  /// Set a breakpoint at a specific address
  BreakSet(u16),
  // Run the emulator until a breakpoint is hit
  Continue,
  ReadMemory(u16),
  ReadMemoryRange(u16, usize),
  ReadRegisters,
  Step,
}

fn normalize_command(token: Option<&str>) -> Option<String> {
  let inner = token?;
  let token_str = String::from_str(inner).ok()?;
  Some(token_str.trim().to_lowercase())
}

pub fn parse_command(line: &str) -> Option<Command> {
  let mut tokens = line.split_whitespace();
  let first_token = normalize_command(tokens.next())?;
  match first_token.as_str() {
    "break" => {
      let addr_str = tokens.next()?;
      let addr = parse_address(addr_str)?;
      Some(Command::BreakSet(addr))
    },
    "c" | "continue" => {
      Some(Command::Continue)
    },

    "info" => {
      let next = normalize_command(tokens.next())?;
      match next.as_str() {
        "reg" | "registers" => {
          Some(Command::ReadRegisters)
        },
        _ => None,
      }
    },

    "p" | "print" => {
      let arg_1 = tokens.next()?;
      let addr = parse_address(arg_1)?;
      Some(Command::ReadMemory(addr))
    },

    "s" | "step" => {
      Some(Command::Step)
    },
  
    _ => None,
  }
}

pub fn parse_address(token: &str) -> Option<u16> {
  let trimmed = token.trim();
  if trimmed.starts_with("0x") {
    return u16::from_str_radix(unsafe { trimmed.get_unchecked(2..) }, 16).ok();
  }
  let addr: Option<u16> = token.trim().parse().ok();
  addr
}

#[cfg(test)]
mod tests {
  use super::{Command, parse_address, parse_command};

  #[test]
  fn parse_stepping() {
    assert_eq!(parse_command("c"), Some(Command::Continue));
    assert_eq!(parse_command(" continue  "), Some(Command::Continue));
    assert_eq!(parse_command("step"), Some(Command::Step));
    assert_eq!(parse_command("s  "), Some(Command::Step));
  }

  #[test]
  fn parse_printing() {
    assert_eq!(parse_command("p 0xff0f"), Some(Command::ReadMemory(0xff0f)));
    assert_eq!(parse_command("print 50"), Some(Command::ReadMemory(50)));
  }

  #[test]
  fn parse_address_arg() {
    assert_eq!(parse_address("0x3020"), Some(0x3020));
    assert_eq!(parse_address("512"), Some(512));
  }

  #[test]
  fn parse_info_command() {
    assert_eq!(parse_command("info registers"), Some(Command::ReadRegisters));
  }
}


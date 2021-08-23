#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use self::windows::WindowShell;
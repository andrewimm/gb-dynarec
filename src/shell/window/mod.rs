#[cfg(windows)]
pub mod windows;
#[cfg(unix)]
pub mod x11;

#[cfg(windows)]
pub use self::windows::WindowShell;
#[cfg(unix)]
pub use self::x11::WindowShell;

pub static WINDOW_TITLE: &str = "GB DYNAREC";

fn main() {
  #[cfg(windows)]
  {
    windows::build! {
      Windows::Win32::System::Memory::{
        CreateFileMappingA,
        MapViewOfFile,
        UnmapViewOfFile,
        VirtualAlloc,
        VirtualFree,
        VirtualProtect,
      },
    };
  }
}
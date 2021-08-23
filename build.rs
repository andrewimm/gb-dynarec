fn main() {
  #[cfg(windows)]
  {
    windows::build! {
      Windows::Win32::Foundation::HWND,
      Windows::Win32::Graphics::Gdi::{
        BitBlt,
        CreateCompatibleBitmap,
        CreateCompatibleDC,
        CreateDIBSection,
        DeleteObject,
        GetDC,
        ReleaseDC,
        SelectObject,
      },
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
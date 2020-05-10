#[cfg(windows)]
#[path = "windows/mod.rs"]
pub mod windows;

#[cfg(windows)]
pub use windows::window::*;
#[cfg(windows)]
pub use windows::monitor::*;

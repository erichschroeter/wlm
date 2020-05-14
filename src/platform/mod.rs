#[cfg(windows)]
#[path = "windows/mod.rs"]
pub mod windows;

#[cfg(windows)]
pub use windows::monitor::*;
#[cfg(windows)]
pub use windows::window::*;

#[cfg(all(not(target_os = "windows")))]
compile_error!("The platform you're compiling for is currently not supported");

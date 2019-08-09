#[cfg(windows)] extern crate winapi;
use std::io::Error;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::windef::HWND;
use winapi::shared::ntdef::WCHAR;
// use winapi::shared::ntdef::LPWSTR;
use winapi::um::winuser::EnumWindows;
use winapi::um::winuser::GetWindowTextW;
use winapi::um::winuser::IsWindowVisible;

#[cfg(windows)]
fn print_message(msg: &str) -> Result<i32, Error> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use winapi::um::winuser::{MB_OK, MessageBoxW};
    let wide: Vec<u16> = OsStr::new(msg).encode_wide().chain(once(0)).collect();
    let ret = unsafe {
        MessageBoxW(null_mut(), wide.as_ptr(), wide.as_ptr(), MB_OK)
    };
    if ret == 0 { Err(Error::last_os_error()) }
    else { Ok(ret) }
}

#[cfg(not(windows))]
fn print_message(msg: &str) -> Result<(), Error> {
    println!("{}", msg);
    Ok(())
}

#[cfg(windows)]
unsafe extern "system" fn window_info_callback(
    hwnd: HWND,
    _l_param: LPARAM
) -> i32 {
    let mut text: [WCHAR; 64] = [0; 64];
    let is_visible = IsWindowVisible(hwnd) != 0;
    if is_visible {
        GetWindowTextW(hwnd, text.as_mut_ptr(), 64);
        println!("{:?}: {}", hwnd, String::from_utf16(&text).unwrap());
    }
    1
}

fn main() {
    unsafe {
        EnumWindows(Some(window_info_callback), 5);
    }
    print_message("Hello, world!").unwrap();
}

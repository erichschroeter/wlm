#[cfg(windows)] extern crate winapi;
use std::io::Error;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::DWORD;
use winapi::shared::minwindef::MAX_PATH;
use winapi::shared::minwindef::HINSTANCE;
use winapi::shared::windef::HWND;
use winapi::shared::ntdef::WCHAR;
use winapi::shared::ntdef::NULL;
use winapi::um::winuser::EnumWindows;
use winapi::um::winuser::GetWindowTextW;
use winapi::um::winuser::IsWindowVisible;
use winapi::um::winuser::GetWindowThreadProcessId;
use winapi::um::winnt::HANDLE;
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::psapi::GetModuleFileNameExW;
use winapi::um::handleapi::CloseHandle;

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
        let mut proc_id: DWORD = 0;
        let mut module_name: [WCHAR; MAX_PATH] = [0; MAX_PATH];
        GetWindowTextW(hwnd, text.as_mut_ptr(), 64);
        GetWindowThreadProcessId(hwnd, &mut proc_id);
        let process_handle: HANDLE = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, proc_id);
        GetModuleFileNameExW(process_handle, NULL as HINSTANCE, module_name.as_mut_ptr(), MAX_PATH as u32);
        CloseHandle(process_handle);
        println!("{:?}: {} ------- {}", hwnd, String::from_utf16(&text).unwrap(), String::from_utf16(&module_name).unwrap());
    }
    1
}

fn main() {
    unsafe {
        EnumWindows(Some(window_info_callback), 5);
    }
    print_message("Hello, world!").unwrap();
}

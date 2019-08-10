#[cfg(windows)] extern crate winapi;
use std::io::Error;
use std::path::Path;
use std::ffi::OsStr;
use winapi::shared::minwindef::LPARAM;
use winapi::shared::minwindef::DWORD;
use winapi::shared::minwindef::MAX_PATH;
use winapi::shared::minwindef::HINSTANCE;
use winapi::shared::windef::RECT;
use winapi::shared::windef::HWND;
use winapi::shared::ntdef::WCHAR;
use winapi::shared::ntdef::NULL;
use winapi::um::winuser::EnumWindows;
use winapi::um::winuser::GetWindowTextW;
use winapi::um::winuser::IsWindowVisible;
use winapi::um::winuser::GetWindowThreadProcessId;
use winapi::um::winuser::GetWindowRect;
use winapi::um::winuser::GetWindowLongPtrW;
use winapi::um::winuser::{GWL_EXSTYLE, GWL_STYLE};
use winapi::um::winuser::WS_EX_APPWINDOW;
use winapi::um::winuser::{WS_EX_WINDOWEDGE, WS_EX_TOOLWINDOW, WS_EX_NOREDIRECTIONBITMAP};
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
    let window_exstyle = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
    let is_visible_on_screen = (window_exstyle & WS_EX_WINDOWEDGE as isize) != 0;
    let is_toolwindow = (window_exstyle & WS_EX_TOOLWINDOW as isize) != 0;
    // let is_noredirectionbitmap = (window_exstyle & WS_EX_NOREDIRECTIONBITMAP as isize) != 0;
    let window_style = GetWindowLongPtrW(hwnd, GWL_STYLE);
    if is_visible && is_visible_on_screen && !is_toolwindow {
        let mut proc_id: DWORD = 0;
        let mut module_name: [WCHAR; MAX_PATH] = [0; MAX_PATH];
        let mut dimensions = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        let dimptr = &mut dimensions as *mut RECT;
        GetWindowTextW(hwnd, text.as_mut_ptr(), 64);
        let mut text = text.to_vec();
        if let Some(first) = text.iter().position(|&b| b == 0) {
            text.truncate(first);
        }
        GetWindowThreadProcessId(hwnd, &mut proc_id);
        GetWindowRect(hwnd, dimptr);
        let process_handle: HANDLE = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, proc_id);
        GetModuleFileNameExW(process_handle, NULL as HINSTANCE, module_name.as_mut_ptr(), MAX_PATH as u32);
        CloseHandle(process_handle);
        let mut module_name = module_name.to_vec();
        if let Some(first) = module_name.iter().position(|&b| b == 0) {
            module_name.truncate(first)
        }
        let process_name = String::from_utf16(&module_name).unwrap();
        let process_path = Path::new(&process_name);
        let process_name = match process_path.file_name() {
            Some(path) => {
                path.to_str().unwrap()
            },
            None => { "" }
        };
        if process_name == "explorer.exe" {
            if !text.is_empty() {
                println!("[{:?}] {}\n\t{}\n\t({}, {})\n\t{}, {}\n\tEX_STYLE: {} (0x{:x})\n\tSTYLE: (0x{:x})",
                    hwnd,
                    String::from_utf16(&text).unwrap(),
                    String::from_utf16(&module_name).unwrap(),
                    dimensions.top,
                    dimensions.bottom,
                    dimensions.left,
                    dimensions.right,
                    is_visible_on_screen,
                    window_exstyle,
                    window_style,
                );
            }
        } else {
            println!("[{:?}] {}\n\t{}\n\t({}, {})\n\t{}, {}\n\tEX_STYLE: {} (0x{:x})\n\tSTYLE: (0x{:x})",
                hwnd,
                String::from_utf16(&text).unwrap(),
                String::from_utf16(&module_name).unwrap(),
                dimensions.top,
                dimensions.bottom,
                dimensions.left,
                dimensions.right,
                is_visible_on_screen,
                window_exstyle,
                window_style,
            );
        }
    }
    1
}

fn main() {
    unsafe {
        EnumWindows(Some(window_info_callback), 5);
    }
}

extern crate clap;
use clap::{Arg, App, SubCommand};
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

const MAX_WINDOW_TITLE: usize = 128;

// TODO list
// [x] Create ls command
// [x]   Implement to print all visible windows
// [ ] Create apply command
// [ ]   Implement to load profile file and apply settings

#[derive(Debug)]
pub struct Location {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug)]
pub struct Dimensions {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug)]
pub struct WindowInfo {
    pub window_title: String,
    pub window_process: String,
    pub location: Location,
    pub dimensions: Dimensions,
}

static mut window_list: Option<Vec<WindowInfo>> = None;

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

fn get_window_title(hwnd: HWND) -> String {
    let mut window_title: [WCHAR; MAX_WINDOW_TITLE] = [0; MAX_WINDOW_TITLE];
    unsafe {
        GetWindowTextW(hwnd, window_title.as_mut_ptr(), MAX_WINDOW_TITLE as i32);
    }
    let mut window_title = window_title.to_vec();
    if let Some(first) = window_title.iter().position(|&b| b == 0) {
        window_title.truncate(first);
    }
    String::from_utf16(&window_title).unwrap()
}

fn get_window_process(hwnd: HWND) -> String {
    let mut proc_id: DWORD = 0;
    let mut window_process: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut proc_id);
        let process_handle: HANDLE = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, proc_id);
        GetModuleFileNameExW(process_handle, NULL as HINSTANCE, window_process.as_mut_ptr(), MAX_PATH as u32);
        CloseHandle(process_handle);
    }
    let mut window_process = window_process.to_vec();
    if let Some(first) = window_process.iter().position(|&b| b == 0) {
        window_process.truncate(first)
    }
    String::from_utf16(&window_process).unwrap()
}

fn basename(path: &str) -> String {
    let path = Path::new(&path);
    let window_process = match path.file_name() {
        Some(path) => {
            path.to_str().unwrap()
        },
        None => { "" }
    };
    window_process.to_owned()
}

fn get_window_dimensions(hwnd: HWND) -> (Location, Dimensions) {
    let mut dimensions = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let dimptr = &mut dimensions as *mut RECT;
    unsafe {
        GetWindowRect(hwnd, dimptr);
    }
    (
        Location {
            x: dimensions.left,
            y: dimensions.top,
        },
        Dimensions {
            width: dimensions.right - dimensions.left,
            height: dimensions.bottom - dimensions.top,
        }
    )
}

#[cfg(windows)]
unsafe extern "system" fn window_info_callback(
    hwnd: HWND,
    _l_param: LPARAM
) -> i32 {
    let is_visible = IsWindowVisible(hwnd) != 0;
    let window_exstyle = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
    let is_visible_on_screen = (window_exstyle & WS_EX_WINDOWEDGE as isize) != 0;
    let is_toolwindow = (window_exstyle & WS_EX_TOOLWINDOW as isize) != 0;
    // let is_noredirectionbitmap = (window_exstyle & WS_EX_NOREDIRECTIONBITMAP as isize) != 0;
    let window_style = GetWindowLongPtrW(hwnd, GWL_STYLE);
    if is_visible && is_visible_on_screen && !is_toolwindow {
        let (location, dimensions) = get_window_dimensions(hwnd);
        let window_title = get_window_title(hwnd);
        let window_process = get_window_process(hwnd);
        let window_process = basename(&window_process);
        if window_process == "explorer.exe" {
            if !window_title.is_empty() {
                println!("[{:?}] {}\n\t{}\n\t({}, {})\n\t{}, {}\n\tEX_STYLE: {} (0x{:x})\n\tSTYLE: (0x{:x})",
                    hwnd,
                    window_title,
                    window_process,
                    location.x,
                    location.y,
                    dimensions.width,
                    dimensions.height,
                    is_visible_on_screen,
                    window_exstyle,
                    window_style,
                );
                match &mut window_list {
                    Some(list) => {
                        list.push(WindowInfo {
                            window_title,
                            window_process,
                            location: Location {
                                x: location.x,
                                y: location.y,
                            },
                            dimensions: Dimensions {
                                width: dimensions.width,
                                height: dimensions.height,
                            }
                        });
                    },
                    None => {
                        window_list = Some(Vec::new());
                    }
                }
            }
        } else {
            println!("[{:?}] {}\n\t{}\n\t({}, {})\n\t{}, {}\n\tEX_STYLE: {} (0x{:x})\n\tSTYLE: (0x{:x})",
                hwnd,
                window_title,
                window_process,
                location.x,
                location.y,
                dimensions.width,
                dimensions.height,
                is_visible_on_screen,
                window_exstyle,
                window_style,
            );
            match &mut window_list {
                Some(list) => {
                    list.push(WindowInfo {
                        window_title,
                        window_process,
                        location: Location {
                            x: location.x,
                            y: location.y,
                        },
                        dimensions: Dimensions {
                            width: dimensions.width,
                            height: dimensions.height,
                        }
                    });
                },
                None => {
                    window_list = Some(Vec::new());
                }
            }
        }
    }
    1
}

fn main() {
    let matches = App::new("window-layout-manager")
        .version("1.0")
        .about("Applies window properties based on profile settings.")
        .arg(Arg::with_name("profile")
            .long("profile")
            .value_name("FILE")
            .help("Sets the profile to be loaded")
            .takes_value(true))
        .subcommand(SubCommand::with_name("ls")
            .about("lists active windows and their properies"))
        .subcommand(SubCommand::with_name("ls")
            .about("lists active windows and their properies"))
        .get_matches();
    if let Some(_matches) = matches.subcommand_matches("ls") {
        unsafe {
            EnumWindows(Some(window_info_callback), 0);
            // TODO is there a way to access window_list in a safe manner?
            match &window_list {
                Some(list) => {
                    for w in list {
                        println!("{:?}", w);
                    }
                },
                None => {}
            }
        }
    }
}

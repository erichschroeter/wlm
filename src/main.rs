extern crate clap;
use clap::{Arg, App, SubCommand};
#[cfg(windows)] extern crate winapi;
use std::fmt;
use std::path::Path;
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
use winapi::um::winuser::GWL_EXSTYLE;
use winapi::um::winuser::{WS_EX_WINDOWEDGE, WS_EX_TOOLWINDOW};
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
    pub hwnd: HWND,
    pub window_title: String,
    pub window_process: String,
    pub location: Location,
    pub dimensions: Dimensions,
}

static mut WINDOW_LIST: Option<Vec<WindowInfo>> = None;

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl fmt::Display for Dimensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} x {}", self.width, self.height)
    }
}

impl fmt::Display for WindowInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}]\n\t\"{}\"\n\t{}\n\t{}\n\t{}", self.hwnd, self.window_title, self.window_process, self.location, self.dimensions)
    }
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

fn add_window(window: WindowInfo) {
    unsafe {
        match &mut WINDOW_LIST {
            Some(list) => {
                list.push(window);
            },
            None => {
                let mut list = Vec::new();
                list.push(window);
                WINDOW_LIST = Some(list);
            }
        }
    }
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
    if is_visible && is_visible_on_screen && !is_toolwindow {
        let (location, dimensions) = get_window_dimensions(hwnd);
        let window_title = get_window_title(hwnd);
        let window_process = get_window_process(hwnd);
        let window_process = basename(&window_process);
        if window_process == "explorer.exe" {
            if !window_title.is_empty() {
                add_window(WindowInfo {
                    hwnd,
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
            }
        } else {
            add_window(WindowInfo {
                hwnd,
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
            // TODO is there a way to access WINDOW_LIST in a safe manner?
            match &WINDOW_LIST {
                Some(list) => {
                    for w in list {
                        println!("{}", w);
                    }
                },
                None => {}
            }
        }
    }
}

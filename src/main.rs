extern crate clap;
use clap::{Arg, App, SubCommand};
use serde::{Serialize, Deserialize};
use std::fmt;
use std::path::Path;
#[cfg(windows)] extern crate winapi;
extern crate regex;
use regex::Regex;
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
use winapi::um::winuser::{WM_NULL, HDWP, BeginDeferWindowPos, DeferWindowPos, EndDeferWindowPos};
use winapi::um::winuser::{SWP_NOZORDER, SWP_NOOWNERZORDER, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOMOVE};
use winapi::um::winnt::HANDLE;
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::psapi::GetModuleFileNameExW;
use winapi::um::handleapi::CloseHandle;
use winapi::um::winbase::FormatMessageW;
use winapi::um::winbase::{FORMAT_MESSAGE_ARGUMENT_ARRAY, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS};
use winapi::um::errhandlingapi::GetLastError;

const MAX_WINDOW_TITLE: usize = 128;

#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dimensions {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Properties {
    #[serde(skip_deserializing)]
    #[serde(skip_serializing)]
    // pub hwnd: HWND,
    pub hwnd: u64,
    pub title: Option<String>,
    pub process: Option<String>,
    pub location: Option<Location>,
    pub dimensions: Option<Dimensions>,
}

#[cfg(windows)]
struct Window {
    handle: HWND,
}

static mut WINDOW_LIST: Option<Vec<Window>> = None;
static mut PROFILE: Option<Vec<Properties>> = None;
static mut G_DEFER_HDWP: Option<HDWP> = None;

trait HasProperties {
    fn properties(&self) -> Properties;
}

impl HasProperties for Window {
    fn properties(&self) -> Properties {
        let title = get_window_title(self.handle);
        let process = get_window_process(self.handle);
        let (location, dimensions) = get_window_dimensions(self.handle);
        Properties {
            hwnd: self.handle as u64,
            title: Some(title),
            process: Some(process),
            location: Some(location),
            dimensions: Some(dimensions),
        }
    }
}

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

impl fmt::Display for Properties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        output.push_str(&format!("[{:?}]", self.hwnd as HWND));
        match &self.title {
            Some(title) => output.push_str(&format!("\n\t\"{}\"", title)),
            None => {}
        }
        match &self.process {
            Some(process) => output.push_str(&format!("\n\t{}", process)),
            None => {}
        }
        match &self.location {
            Some(location) => output.push_str(&format!("\n\t{}", location)),
            None => {}
        }
        match &self.dimensions {
            Some(dimensions) => output.push_str(&format!("\n\t{}", dimensions)),
            None => {}
        }
        write!(f, "{}", output)
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

#[cfg(windows)]
fn get_last_error_message() -> String {
    let mut error_code = 0;

    unsafe {
        error_code = GetLastError();
    }

    if error_code == 0 {
        String::from("")
    } else {
        let mut v = [0u16; 255];
        unsafe {
            let msg_size = FormatMessageW(
                FORMAT_MESSAGE_ARGUMENT_ARRAY | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
                std::ptr::null(),
                error_code,
                0,
                v.as_mut_ptr(),
                255,
                std::ptr::null_mut());
            if msg_size == 0 {
                String::from("")
            } else {
                String::from_utf16(&v).unwrap()
            }
        }
    }
}

fn get_window_process(hwnd: HWND) -> String {
    let mut proc_id: DWORD = 0;
    let mut window_process: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut proc_id);
        let process_handle: HANDLE = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ, 0, proc_id);
        let mut buf_size: DWORD = MAX_PATH as u32;
        let return_value = GetModuleFileNameExW(process_handle, NULL as HINSTANCE, window_process.as_mut_ptr(), MAX_PATH as u32);
        // let return_value = QueryFullProcessImageNameW(process_handle, 0, window_process.as_mut_ptr(), &mut buf_size);
        if return_value == 0 {
            // let mut error_message: [WCHAR; 256] = ['\0' as u16; 256];
            // let e = GetLastError();
            // eprintln!("QueryFullProcessImageNameW Failed: {}", e);
            let msg = get_last_error_message();
            eprintln!("QueryFullProcessImageNameW Failed: {}", msg);
        }
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
fn apply_profile_properties(hdwp: &mut HDWP, hwnd: HWND, properties: &Properties) {
    let mut location = Location { x: 0, y: 0 };
    let mut dimensions = Dimensions { width: 0, height: 0 };
    let mut flags = SWP_NOZORDER | SWP_NOOWNERZORDER | SWP_NOACTIVATE;
    match &properties.location {
        Some(new_location) => {
            location.x = new_location.x;
            location.y = new_location.y;
        },
        None => flags |= SWP_NOMOVE,
    }
    match &properties.dimensions {
        Some(new_dimensions) => {
            dimensions.width = new_dimensions.width;
            dimensions.height = new_dimensions.height;
        },
        None => flags |= SWP_NOSIZE,
    }
    unsafe {
        *hdwp = DeferWindowPos(
            *hdwp,
            hwnd,
            WM_NULL as HWND,
            location.x,
            location.y,
            dimensions.width,
            dimensions.height,
            flags);
    }
}

#[cfg(windows)]
unsafe extern "system" fn apply_profile_callback(
    hwnd: HWND,
    _l_param: LPARAM
) -> i32 {
    match &PROFILE {
        Some(profile) => {
            let window = Window { handle: hwnd };
            let properties = window.properties();
            for profile_window in profile {
                match &profile_window.title {
                    Some(profile_title) => {
                        let re = Regex::new(profile_title);
                        match re {
                            Ok(re) => {
                                match &properties.title {
                                    Some(hwnd_title) => {
                                        if re.is_match(hwnd_title) {
                                            match G_DEFER_HDWP {
                                                Some(mut hdwp) => apply_profile_properties(&mut hdwp, hwnd, profile_window),
                                                None => eprintln!("BeginDeferWindowPos was not called before DeferWindowPos"),
                                            }
                                        } else {
                                            // eprintln!("'{}' did not match", hwnd_title)
                                        }
                                    },
                                    None => {}
                                }
                            },
                            Err(e) => { eprintln!("Invalid regex: {}", e) }
                        }
                    },
                    None => {
                        match &profile_window.process {
                            Some(profile_process) => {
                                let re = Regex::new(profile_process);
                                match re {
                                    Ok(re) => {
                                        match &properties.process {
                                            Some(hwnd_process) => {
                                                if re.is_match(hwnd_process) {
                                                    match G_DEFER_HDWP {
                                                        Some(mut hdwp) => apply_profile_properties(&mut hdwp, hwnd, profile_window),
                                                        None => eprintln!("BeginDeferWindowPos was not called before DeferWindowPos"),
                                                    }
                                                } else {
                                                    // eprintln!("'{}' did not match", hwnd_process)
                                                }
                                            },
                                            None => {}
                                        }
                                    },
                                    Err(e) => { eprintln!("Invalid regex: {}", e) }
                                }
                            },
                            None => {}
                        }
                    }
                }
            }
        },
        None => {}
    }
    1
}

use winapi::um::winnt::SE_PRIVILEGE_ENABLED;
use winapi::um::winnt::SE_PROF_SINGLE_PROCESS_NAME;
use winapi::um::winnt::TOKEN_PRIVILEGES;
use winapi::um::winnt::LUID_AND_ATTRIBUTES;
use winapi::um::winnt::LPCWSTR;
use winapi::um::winnt::TOKEN_ALL_ACCESS;
use winapi::um::winnt::PTOKEN_PRIVILEGES;
use winapi::um::winnt::PHANDLE;
use winapi::shared::ntdef::LUID;
use winapi::shared::minwindef::PDWORD;
use winapi::um::winbase::LookupPrivilegeValueW;
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::processthreadsapi::OpenProcessToken;
use winapi::um::securitybaseapi::AdjustTokenPrivileges;
use winapi::um::errhandlingapi::GetLastError;

#[cfg(windows)]
fn enable_windows_privilege(privilege: &str) -> bool {
    let mut luid = LUID {
        LowPart: 0,
        HighPart: 0,
    };
    let mut tp = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [
            LUID_AND_ATTRIBUTES {
                Luid: luid,
                Attributes: SE_PRIVILEGE_ENABLED,
            }
        ]
    };
    unsafe {
        let mut current_process = GetCurrentProcess();
        let mut current_token: HANDLE = NULL;
        let result = LookupPrivilegeValueW(NULL as LPCWSTR, privilege.as_ptr() as LPCWSTR, &mut luid);
        if result == 0 {
            let e = GetLastError();
            eprintln!("LookupPrivilegeValueW failed: {}", e);
            return false;
        }
        let result = OpenProcessToken(current_process, TOKEN_ALL_ACCESS, current_token as PHANDLE);
        if result == 0 {
            eprintln!("OpenProcessToken failed");
            return false;
        }
        let result = AdjustTokenPrivileges(current_token, 0, &mut tp, std::mem::size_of::<TOKEN_PRIVILEGES>() as u32, NULL as PTOKEN_PRIVILEGES, NULL as PDWORD);
        if result == 0 {
            eprintln!("AdjustTokenPrivileges failed");
            return false;
        }
        true
    }
}

#[cfg(windows)]
unsafe extern "system" fn filter_windows_callback(
    hwnd: HWND,
    _l_param: LPARAM
) -> i32 {
    let is_visible = IsWindowVisible(hwnd) != 0;
    let window_exstyle = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
    let is_visible_on_screen = (window_exstyle & WS_EX_WINDOWEDGE as isize) != 0;
    let is_toolwindow = (window_exstyle & WS_EX_TOOLWINDOW as isize) != 0;
    // enable_windows_privilege(SE_PROF_SINGLE_PROCESS_NAME);
    if is_visible && is_visible_on_screen && !is_toolwindow {
        let window_title = get_window_title(hwnd);
        let window_process = get_window_process(hwnd);
        let window_process = basename(&window_process);
        // There are typically a lot of explorer.exe "windows" that get listed that don't have a UI,
        // so filter them out to avoid clutter.
        if window_process == "explorer.exe" {
            if !window_title.is_empty() {
                match &mut WINDOW_LIST {
                    Some(list) => list.push(Window { handle: hwnd }),
                    None => {
                        WINDOW_LIST = Some(Vec::new());
                        match &mut WINDOW_LIST {
                            Some(list) => list.push(Window { handle: hwnd }),
                            None => {}
                        }
                    }
                }
            }
        } else {
            match &mut WINDOW_LIST {
                Some(list) => list.push(Window { handle: hwnd }),
                None => {
                    WINDOW_LIST = Some(Vec::new());
                    match &mut WINDOW_LIST {
                        Some(list) => list.push(Window { handle: hwnd }),
                        None => {}
                    }
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
        .subcommand(SubCommand::with_name("ls")
            .about("Lists active windows and their properies.

    [HWND - Window Handle]
        Title
        Process
        (x, y) top-left of window
        (width, height)")
            .arg(Arg::with_name("as-json")
                .help("Change output format of active windows in JSON")
                .long("as-json")))
        .subcommand(SubCommand::with_name("apply")
            .about("apply a profile to active windows")
            .arg(Arg::with_name("PROFILE")
                .help("Sets the profile to be loaded")
                .required(true)
                .index(1)))
        .get_matches();
    match matches.subcommand() {
        ("ls", Some(matches)) => {
            unsafe {
                EnumWindows(Some(filter_windows_callback), 0);
                // TODO is there a way to access WINDOW_LIST in a safe manner?
                match &WINDOW_LIST {
                    Some(list) => {
                        if matches.is_present("as-json") {
                            let mut window_list = Vec::new();
                            for item in list {
                                window_list.push(item.properties());
                            } 
                            print!("{}", serde_json::to_string_pretty(&window_list).unwrap_or_default());
                        } else {
                            for window in list {
                                println!("{}", window.properties());
                            }
                        }
                    },
                    None => {}
                }
            }
        },
        ("apply", Some(matches)) => {
            let profile = matches.value_of("PROFILE").unwrap();
            let json = std::fs::read_to_string(profile).expect("Failed reading profile");
            unsafe {
                PROFILE = Some(serde_json::from_str(&json).unwrap());
                G_DEFER_HDWP = Some(BeginDeferWindowPos(1));
                EnumWindows(Some(apply_profile_callback), 0);
                match G_DEFER_HDWP {
                    Some(hdwp) => {
                        if hdwp != NULL {
                            EndDeferWindowPos(hdwp);
                        }
                    },
                    None => {}
                }
            }
        }
        _ => {}
    }
}

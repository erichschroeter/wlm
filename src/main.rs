extern crate clap;
use clap::{Arg, App, SubCommand};
use serde::{Serialize, Deserialize};
use std::convert::From;
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
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::psapi::GetModuleFileNameExW;
use winapi::um::handleapi::CloseHandle;
use winapi::um::winbase::FormatMessageW;
use winapi::um::winbase::{FORMAT_MESSAGE_ARGUMENT_ARRAY, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS};
use winapi::um::errhandlingapi::GetLastError;

const MAX_WINDOW_TITLE: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimensions {
    pub width: i32,
    pub height: i32,
}

fn default_as_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    #[serde(skip_deserializing)]
    #[serde(skip_serializing)]
    // pub hwnd: HWND,
    pub hwnd: Vec<u64>,
    #[serde(default = "default_as_true")]
    pub allow_cascade: bool,
    pub title: Option<String>,
    pub process: Option<String>,
    pub location: Option<Location>,
    pub dimensions: Option<Dimensions>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Profile {
    windows: Vec<Window>,
}

impl Profile {
    fn new() -> Profile {
        Profile {
            windows: Vec::new(),
        }
    }
}

static mut ACTIVE_WINDOWS: Option<Vec<Window>> = None;
static mut G_DEFER_HDWP: Option<HDWP> = None;

impl From<HWND> for Window {
    fn from(item: HWND) -> Self {
        let title = get_window_title(item);
        let process = get_window_process(item);
        let (location, dimensions) = get_window_dimensions(item);
        Window {
            hwnd: vec![item as u64],
            allow_cascade: true,
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

impl fmt::Display for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        output.push_str(&format!("{:?}", (&self.hwnd).into_iter().map(|i| format!("0x{:x}", i)).collect::<Vec<String>>()));
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

#[allow(dead_code)]
#[cfg(windows)]
fn get_last_error_message() -> String {
    #[allow(unused_assignments)]
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
fn check_valid_window(hwnd: HWND) -> Option<Window> {
    #[allow(unused_assignments)]
    let mut is_visible = false;
    #[allow(unused_assignments)]
    let mut window_exstyle = 0;
    unsafe {
        is_visible = IsWindowVisible(hwnd) != 0;
        window_exstyle = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
    }
    let is_visible_on_screen = (window_exstyle & WS_EX_WINDOWEDGE as isize) != 0;
    let is_toolwindow = (window_exstyle & WS_EX_TOOLWINDOW as isize) != 0;
    if is_visible && is_visible_on_screen && !is_toolwindow {
        let window_title = get_window_title(hwnd);
        let window_process = get_window_process(hwnd);
        let window_process = basename(&window_process);
        // There are typically a lot of explorer.exe "windows" that get listed
        // that don't have a UI, so filter them out to avoid clutter.
        if window_process == "explorer.exe" {
            if !window_title.is_empty() {
                return Some(hwnd.into());
            }
        } else {
            return Some(hwnd.into());
        }
    }
    None
}

#[cfg(windows)]
fn apply_profile_properties(hdwp: &mut HDWP, hwnd: HWND, window: &Window) {
    let mut location = Location { x: 0, y: 0 };
    let mut dimensions = Dimensions { width: 0, height: 0 };
    /*
     * 0x0020 | SWP_DRAWFRAME      | Draws a frame (defined in the window's class
     *        |                    | description) around the window.
     * 0x0020 | SWP_FRAMECHANGED   | Sends a WM_NCCALCSIZE message to the window,
     *        |                    | even if the window's size is not being changed.
     *        |                    | If this flag is not specified, WM_NCCALCSIZE
     *        |                    | is sent only when the window's size is being changed.
     * 0x0080 | SWP_HIDEWINDOW     | Hides the window.
     * 0x0010 | SWP_NOACTIVATE     | Does not activate the window. If this flag is
     *        |                    | not set, the window is activated and moved to
     *        |                    | the top of either the topmost or non-topmost
     *        |                    | group (depending on the setting of the hWndInsertAfter parameter).
     * 0x0100 | SWP_NOCOPYBITS     | Discards the entire contents of the client area.
     *        |                    | If this flag is not specified, the valid contents
     *        |                    | of the client area are saved and copied back into
     *        |                    | the client area after the window is sized or repositioned.
     * 0x0002 | SWP_NOMOVE         | Retains the current position (ignores the x and y parameters).
     * 0x0200 | SWP_NOOWNERZORDER  | Does not change the owner window's position in the Z order.
     * 0x0008 | SWP_NOREDRAW       | Does not redraw changes. If this flag is set,
     *        |                    | no repainting of any kind occurs. This applies
     *        |                    | to the client area, the nonclient area (including
     *        |                    | the title bar and scroll bars), and any part of
     *        |                    | the parent window uncovered as a result of the
     *        |                    | window being moved. When this flag is set, the
     *        |                    | application must explicitly invalidate or redraw
     *        |                    | any parts of the window and parent window that need redrawing.
     * 0x0200 | SWP_NOREPOSITION   | Same as the SWP_NOOWNERZORDER flag.
     * 0x0400 | SWP_NOSENDCHANGING | Prevents the window from receiving the WM_WINDOWPOSCHANGING message.
     * 0x0001 | SWP_NOSIZE         | Retains the current size (ignores the cx and cy parameters).
     * 0x0004 | SWP_NOZORDER       | Retains the current Z order (ignores the hWndInsertAfter parameter).
     * 0x0040 | SWP_SHOWWINDOW     | Displays the window.
     */
    let mut flags = SWP_NOZORDER | SWP_NOOWNERZORDER | SWP_NOACTIVATE;
    match &window.location {
        Some(new_location) => {
            location.x = new_location.x;
            location.y = new_location.y;
        },
        None => flags |= SWP_NOMOVE,
    }
    match &window.dimensions {
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

use std::collections::HashMap;

#[cfg(windows)]
fn apply_profile(profile: &Profile) {
    let mut cascade_map: HashMap<String, Location> = HashMap::new();
    #[allow(unused_assignments)]
    let mut hdwp = NULL;
    unsafe {
        hdwp = BeginDeferWindowPos(1);
    }

    for window in &profile.windows {
        for hwnd in &window.hwnd {
            if window.allow_cascade && window.location.is_some() {
                let process = window.process.clone().unwrap();
                match cascade_map.get_mut(&process) {
                    Some(mut location) => {
                        location.x += 30;
                        location.y += 30;
                        let mut window_clone = window.clone();
                        window_clone.location = Some(location.to_owned());
                        apply_profile_properties(&mut hdwp, *hwnd as HWND, &window_clone);
                    },
                    None => {
                        cascade_map.insert(process, window.location.clone().unwrap());
                        apply_profile_properties(&mut hdwp, *hwnd as HWND, &window);
                    }
                }
            } else {
                apply_profile_properties(&mut hdwp, *hwnd as HWND, &window);
            }
        }
    }

    if hdwp != NULL {
        unsafe {
            EndDeferWindowPos(hdwp);
        }
    }
}

#[cfg(windows)]
unsafe extern "system" fn filter_windows_callback(
    hwnd: HWND,
    _l_param: LPARAM
) -> i32 {
    match check_valid_window(hwnd) {
        Some(window) => {
            match &mut ACTIVE_WINDOWS {
                Some(active_windows) => active_windows.push(window),
                None => {
                    ACTIVE_WINDOWS = Some(Vec::new());
                    match &mut ACTIVE_WINDOWS {
                        Some(active_windows) => active_windows.push(window),
                        None => {}
                    }
                }
            }
        },
        None => {}
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
                match &ACTIVE_WINDOWS {
                    Some(active_windows) => {
                        if matches.is_present("as-json") {
                            print!("{}", serde_json::to_string_pretty(&active_windows).unwrap_or_default());
                        } else {
                            for window in active_windows {
                                println!("{}", window);
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
            #[allow(unused_assignments)]
            let mut active_windows = None;
            unsafe {
                EnumWindows(Some(filter_windows_callback), 0);
                active_windows = ACTIVE_WINDOWS.clone();
            }
            match active_windows {
                Some(active_windows) => {
                    let profile: serde_json::Result<Profile> = serde_json::from_str(&json);
                    match profile {
                        Ok(mut profile) => {
                            for active_window in &active_windows {
                                // Find a match in our user-defined profile for the active window.
                                // If a match cannot be found, then simply ignore.
                                // If a match is found, assign the HWND to the user-defined profile attribute.

                                let mut match_found = false;
                                for profile_window in &mut profile.windows {
                                    match &profile_window.title {
                                        Some(profile_title) => {
                                            let re = Regex::new(profile_title);
                                            match re {
                                                Ok(re) => {
                                                    match &active_window.title {
                                                        Some(hwnd_title) => {
                                                            if re.is_match(hwnd_title) {
                                                                profile_window.hwnd.extend(&active_window.hwnd);
                                                                match_found = true;
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
                                                            match &active_window.process {
                                                                Some(hwnd_process) => {
                                                                    if re.is_match(hwnd_process) {
                                                                        profile_window.hwnd.extend(&active_window.hwnd);
                                                                        match_found = true;
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
                                    if match_found {
                                        break;
                                    }
                                }

                            }
                            apply_profile(&profile);
                        },
                        Err(e) => eprintln!("Invalid profile: {}", e)
                    }
                },
                None => {}
            }
        }
        _ => {}
    }
}

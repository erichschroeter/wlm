#[cfg(windows)]
use crate::MAX_WINDOW_TITLE_LENGTH;
use crate::{config::Config, shrink};

use prettytable::{color, format, Attr, Cell, Row, Table};
use std::path::Path;
use std::ptr;
use winapi::{
	shared::{
		minwindef::{BOOL, DWORD, HINSTANCE, LPARAM, MAX_PATH, TRUE},
		ntdef::{NULL, WCHAR},
		windef::{HDC, HMONITOR, LPRECT},
		windef::{HWND, RECT},
	},
	um::{
		dwmapi::{DwmGetWindowAttribute, DWMWA_CLOAKED},
		handleapi::CloseHandle,
		processthreadsapi::OpenProcess,
		psapi::GetModuleFileNameExW,
		winnt::{HANDLE, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
		winuser::{
			self, BeginDeferWindowPos, DeferWindowPos, EndDeferWindowPos, EnumWindows,
			GetWindowLongPtrW, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId,
			IsWindowVisible, GWL_EXSTYLE, HDWP, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER,
			SWP_NOSIZE, SWP_NOZORDER, WM_NULL, WS_EX_TOOLWINDOW, WS_EX_WINDOWEDGE,
		},
	},
};

unsafe extern "system" fn monitor_enum_proc(
	hmonitor: HMONITOR,
	_hdc: HDC,
	_place: LPRECT,
	data: LPARAM,
) -> BOOL {
	let monitors = data as *mut Vec<Monitor>;
	(*monitors).push(Monitor::new(hmonitor));
	TRUE // continue enumeration
}

pub fn list_monitors() -> Vec<Monitor> {
	let mut monitors: Vec<Monitor> = Vec::new();
	unsafe {
		winuser::EnumDisplayMonitors(
			ptr::null_mut(),
			ptr::null_mut(),
			Some(monitor_enum_proc),
			&mut monitors as *mut _ as LPARAM,
		);
	}
	monitors
}

#[derive(Debug)]
pub struct Monitor(HMONITOR);

impl Monitor {
	pub(crate) fn new(hmonitor: HMONITOR) -> Self {
		Monitor(hmonitor)
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowState {
	pub hwnd: HWND,
	pub cascade: bool,
	pub title: Option<String>,
	pub process: Option<String>,
	pub x: Option<i32>,
	pub y: Option<i32>,
	pub w: Option<i32>,
	pub h: Option<i32>,
}

impl std::fmt::Display for WindowState {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:#?}", &self)
	}
}

impl WindowState {
	pub fn new() -> Self {
		WindowState {
			hwnd: 0 as HWND,
			cascade: true,
			title: None,
			process: None,
			x: None,
			y: None,
			w: None,
			h: None,
		}
	}
}

#[derive(Debug)]
struct Windows<'a> {
	config: Option<&'a Config>,
	list: Vec<WindowState>,
}

pub fn list_windows<'a>(config: Option<&'a Config>) -> Option<Vec<WindowState>> {
	let mut windows_state = Windows {
		config: config,
		list: Vec::new(),
	};
	let struct_ptr = &mut windows_state as *mut Windows;
	unsafe {
		EnumWindows(Some(filter_windows_callback), struct_ptr as LPARAM);
	}
	Some(windows_state.list)
}

pub fn print_windows<'a>(config: Option<&'a Config>) {
	let mut table = Table::new();
	if let Some(windows) = list_windows(config) {
		table.set_format(*format::consts::FORMAT_NO_COLSEP);
		table.add_row(Row::new(vec![
			Cell::new("Title").style_spec("c"),
			Cell::new("Process").style_spec("c"),
			Cell::new("Position").style_spec("c"),
			Cell::new("Dimension").style_spec("c"),
		]));
		for w in windows {
			table.add_row(Row::new(vec![
				Cell::new(&shrink(w.title.as_ref().unwrap(), 32))
					.with_style(Attr::ForegroundColor(color::RED)),
				Cell::new(&shrink(w.process.as_ref().unwrap(), 64))
					.with_style(Attr::ForegroundColor(color::GREEN)),
				Cell::new(&format!(
					"({}, {})",
					w.x.as_ref().unwrap(),
					w.y.as_ref().unwrap()
				)),
				Cell::new(&format!(
					"{} x {}",
					w.w.as_ref().unwrap(),
					w.h.as_ref().unwrap()
				)),
			]));
		}
	}
	table.printstd();
}

pub fn layout_windows<'a>(config: Option<&'a Config>) {
	if let Some(config) = config {
		if let Some(mut windows) = list_windows(None) {
			apply_config(config, &mut windows);
		}
	}
}

impl From<HWND> for WindowState {
	fn from(item: HWND) -> Self {
		let title = get_window_title(item);
		let process = get_window_process(item);
		let (x, y, w, h) = get_window_dimensions(item);
		WindowState {
			hwnd: item,
			cascade: true,
			title: Some(title),
			process: Some(process),
			x: Some(x),
			y: Some(y),
			w: Some(w),
			h: Some(h),
		}
	}
}

fn basename(path: &str) -> String {
	let path = Path::new(&path);
	let window_process = match path.file_name() {
		Some(path) => path.to_str().unwrap(),
		None => "",
	};
	window_process.to_owned()
}

fn get_window_process(hwnd: HWND) -> String {
	let mut proc_id: DWORD = 0;
	let mut window_process: [WCHAR; MAX_PATH] = [0; MAX_PATH];
	unsafe {
		GetWindowThreadProcessId(hwnd, &mut proc_id);
		let process_handle: HANDLE =
			OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, proc_id);
		GetModuleFileNameExW(
			process_handle,
			NULL as HINSTANCE,
			window_process.as_mut_ptr(),
			MAX_PATH as u32,
		);
		CloseHandle(process_handle);
	}
	let mut window_process = window_process.to_vec();
	if let Some(first) = window_process.iter().position(|&b| b == 0) {
		window_process.truncate(first)
	}
	String::from_utf16(&window_process).unwrap()
}

fn get_window_title(hwnd: HWND) -> String {
	let mut window_title: [WCHAR; MAX_WINDOW_TITLE_LENGTH] = [0; MAX_WINDOW_TITLE_LENGTH];
	unsafe {
		GetWindowTextW(
			hwnd,
			window_title.as_mut_ptr(),
			MAX_WINDOW_TITLE_LENGTH as i32,
		);
	}
	let mut window_title = window_title.to_vec();
	if let Some(first) = window_title.iter().position(|&b| b == 0) {
		window_title.truncate(first);
	}
	String::from_utf16(&window_title).unwrap()
}

fn get_window_dimensions(hwnd: HWND) -> (i32, i32, i32, i32) {
	let mut dimensions = RECT {
		left: 0,
		top: 0,
		right: 0,
		bottom: 0,
	};
	let dimptr = &mut dimensions as *mut RECT;
	unsafe {
		GetWindowRect(hwnd, dimptr);
	}
	(
		dimensions.left,
		dimensions.top,
		dimensions.right - dimensions.left,
		dimensions.bottom - dimensions.top,
	)
}

fn check_valid_window(hwnd: HWND) -> Option<WindowState> {
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
	if is_visible
		&& is_visible_on_screen
		&& !is_toolwindow
		&& !is_invisible_win10_background_app_window(hwnd)
	{
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

// See https://stackoverflow.com/questions/32149880/how-to-identify-windows-10-background-store-processes-that-have-non-displayed-wi
fn is_invisible_win10_background_app_window(hwnd: HWND) -> bool {
	let mut cloaked_value: u32 = 0;
	unsafe {
		let my_ptr = &mut cloaked_value as *mut u32;
		DwmGetWindowAttribute(
			hwnd,
			DWMWA_CLOAKED,
			my_ptr as *mut winapi::ctypes::c_void,
			std::mem::size_of::<u32>() as u32,
		);
		if cloaked_value != 0 {
			true
		} else {
			false
		}
	}
}

pub struct Position {
	pub x: i32,
	pub y: i32,
}

pub struct Dimensions {
	pub width: i32,
	pub height: i32,
}

fn apply_profile_properties(hdwp: &mut HDWP, window: &WindowState) {
	let mut pos = Position { x: 0, y: 0 };
	let mut dim = Dimensions {
		width: 0,
		height: 0,
	};
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
	if window.x.is_none() && window.y.is_none() {
		flags |= SWP_NOMOVE;
	} else {
		if let Some(x) = window.x {
			pos.x = x;
		}
		if let Some(y) = window.y {
			pos.y = y;
		}
	}
	if window.w.is_none() && window.h.is_none() {
		flags |= SWP_NOSIZE;
	} else {
		if let Some(w) = window.w {
			dim.width = w;
		}
		if let Some(h) = window.h {
			dim.height = h;
		}
	}
	unsafe {
		*hdwp = DeferWindowPos(
			*hdwp,
			window.hwnd,
			WM_NULL as HWND,
			pos.x,
			pos.y,
			dim.width,
			dim.height,
			flags,
		);
	}
}

fn apply_config(config: &Config, windows: &mut [WindowState]) {
	#[allow(unused_assignments)]
	let mut hdwp = NULL;
	unsafe {
		hdwp = BeginDeferWindowPos(1);
	}

	for ms_win in windows {
		if let Some(cfg_win) = config.search(ms_win) {
			ms_win.x = cfg_win.x;
			ms_win.y = cfg_win.y;
			ms_win.w = cfg_win.w;
			ms_win.h = cfg_win.h;
			apply_profile_properties(&mut hdwp, ms_win);
		}
	}

	if hdwp != NULL {
		unsafe {
			EndDeferWindowPos(hdwp);
		}
	}
}

unsafe extern "system" fn filter_windows_callback(hwnd: HWND, l_param: LPARAM) -> i32 {
	let windows_struct = l_param as *mut Windows;
	match check_valid_window(hwnd) {
		Some(window) => (*windows_struct).list.push(window),
		None => {}
	}
	1
}

#[cfg(test)]
mod tests {
	mod apply_config {
		use super::super::*;
		use crate::config::ConfigBuilder;
		use crate::window::WindowBuilder;

		#[test]
		fn window_state_x_updated_to_matching_config() {
			let w1 = WindowBuilder::default()
				.title(Some("Window 1".to_string()))
				.x(Some(100))
				.build()
				.unwrap();
			let config = ConfigBuilder::default().windows(vec![w1]).build().unwrap();
			let mut ws1 = WindowState::new();
			ws1.title = Some("Window 1".to_string());
			ws1.x = Some(0);
			let mut windows = vec![ws1];
			apply_config(&config, &mut windows);
			let ws1 = windows.pop().unwrap();
			assert_eq!(100, ws1.x.unwrap());
		}

		#[test]
		fn window_state_y_updated_to_matching_config() {
			let w1 = WindowBuilder::default()
				.title(Some("Window 1".to_string()))
				.y(Some(100))
				.build()
				.unwrap();
			let config = ConfigBuilder::default().windows(vec![w1]).build().unwrap();
			let mut ws1 = WindowState::new();
			ws1.title = Some("Window 1".to_string());
			ws1.y = Some(0);
			let mut windows = vec![ws1];
			apply_config(&config, &mut windows);
			let ws1 = windows.pop().unwrap();
			assert_eq!(100, ws1.y.unwrap());
		}

		#[test]
		fn window_state_w_updated_to_matching_config() {
			let w1 = WindowBuilder::default()
				.title(Some("Window 1".to_string()))
				.w(Some(100))
				.build()
				.unwrap();
			let config = ConfigBuilder::default().windows(vec![w1]).build().unwrap();
			let mut ws1 = WindowState::new();
			ws1.title = Some("Window 1".to_string());
			ws1.w = Some(0);
			let mut windows = vec![ws1];
			apply_config(&config, &mut windows);
			let ws1 = windows.pop().unwrap();
			assert_eq!(100, ws1.w.unwrap());
		}

		#[test]
		fn window_state_h_updated_to_matching_config() {
			let w1 = WindowBuilder::default()
				.title(Some("Window 1".to_string()))
				.h(Some(100))
				.build()
				.unwrap();
			let config = ConfigBuilder::default().windows(vec![w1]).build().unwrap();
			let mut ws1 = WindowState::new();
			ws1.title = Some("Window 1".to_string());
			ws1.h = Some(0);
			let mut windows = vec![ws1];
			apply_config(&config, &mut windows);
			let ws1 = windows.pop().unwrap();
			assert_eq!(100, ws1.h.unwrap());
		}
	}
}

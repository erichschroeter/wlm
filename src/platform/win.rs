use crate::{Point, WindowProvider};

use crate::layout::{
	Layout, Screen, ScreenBuilder, Window, WindowBuilder, MAX_WINDOW_TITLE_LENGTH,
};

#[cfg(windows)]
use crate::platform::Win32Window as PlatformWindow;

use regex::Regex;
use std::collections::HashMap;
use std::ffi::OsString;
use std::mem;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use std::str::FromStr;
use winapi::shared::minwindef::{DWORD, HINSTANCE, LPARAM, MAX_PATH, TRUE};
use winapi::shared::ntdef::NULL;
use winapi::shared::ntdef::WCHAR;
use winapi::shared::windef::RECT;
use winapi::shared::windef::{HDC, HMONITOR, HWND, LPRECT};
use winapi::um::dwmapi::{DwmGetWindowAttribute, DWMWA_CLOAKED};
// use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::psapi::GetModuleFileNameExW;
// use winapi::um::winbase::{
// 	FormatMessageW, FORMAT_MESSAGE_ARGUMENT_ARRAY, FORMAT_MESSAGE_FROM_SYSTEM,
// 	FORMAT_MESSAGE_IGNORE_INSERTS,
// };
use winapi::um::winnt::HANDLE;
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use winapi::um::winuser::{
	BeginDeferWindowPos, DeferWindowPos, EndDeferWindowPos, EnumDisplayMonitors, EnumWindows,
	GetMonitorInfoW, GetWindowLongPtrW, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId,
	IsWindowVisible, MonitorFromWindow, ShowWindow, GWL_EXSTYLE, HDWP, MONITORINFOEXW,
	MONITOR_DEFAULTTOPRIMARY, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE,
	SWP_NOZORDER, SW_SHOWMAXIMIZED, SW_SHOWMINIMIZED, WM_NULL, WS_EX_TOOLWINDOW, WS_EX_WINDOWEDGE,
};

pub struct Rectangle(RECT);

impl Rectangle {
	pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
		Rectangle {
			0: RECT {
				left: x,
				top: y,
				right: x + width,
				bottom: y + height,
			},
		}
	}
	/// Returns the upper left point of the `Rectangle`.
	pub fn origin(&self) -> Point {
		Point {
			x: self.0.left,
			y: self.0.top,
		}
	}
	pub fn width(&self) -> i32 {
		self.0.right - self.0.left
	}

	pub fn height(&self) -> i32 {
		self.0.bottom - self.0.top
	}
}

impl Default for Rectangle {
	fn default() -> Self {
		Rectangle::new(0, 0, 0, 0)
	}
}

impl std::fmt::Display for Rectangle {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"[{}x{}] @ {}",
			self.width(),
			self.height(),
			self.origin()
		)
	}
}

impl From<Point> for Rectangle {
	fn from(p: Point) -> Self {
		Rectangle::new(p.x, p.y, 0, 0)
	}
}

impl From<RECT> for Rectangle {
	fn from(value: RECT) -> Self {
		Rectangle::new(
			value.top,
			value.left,
			value.left - value.right,
			value.top - value.bottom,
		)
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Win32Window {
	pub hwnd: HWND,
	pub monitor: HMONITOR,
	pub window: Window,
}

impl std::fmt::Display for Win32Window {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:#?}", &self)
	}
}

/// Converts a string representation of a size into pixels.
///
/// This function takes a size value as a string and a reference to a `Rectangle` representing
/// monitor information. The size can be specified in either percentage or pixels. If a
/// percentage is provided, it calculates the size relative to the width of the monitor
/// specified in `monitor_info`. If a pixel value is provided, it returns that value directly.
///
/// # Arguments
/// * `value` - A string slice that holds the size to be converted. This can be in the format
///   of a percentage (like "50%") or a pixel count (like "200px" or "200").
/// * `monitor_info` - A reference to a `Rectangle` struct representing the dimensions of the
///   monitor. This is used to calculate the pixel value when a percentage is provided.
///
/// # Returns
/// An `i32` representing the size in pixels. If the input string is not in a recognized format, it returns 0.
///
/// # Examples
/// ```
/// # use wlm::platform::win::{into_pixels, Rectangle};
/// let monitor = Rectangle::new(0, 0, 1920, 1080); // Example monitor dimensions
/// assert_eq!(into_pixels("50%", &monitor), 960);  // 50% of 1920
/// assert_eq!(into_pixels("200px", &monitor), 200); // Explicit pixel value
/// ```
///
/// # Errors
/// This function panics if the regex expressions fail to compile, which is unlikely in normal usage.
pub fn into_pixels<S: AsRef<str>>(value: S, monitor_info: &Rectangle) -> i32 {
	let percent_regex = Regex::new(r"(?P<percent>\d+)%").unwrap();
	if let Some(percent) = percent_regex.captures(value.as_ref()) {
		if let Ok(percent) = f64::from_str(percent.name("percent").unwrap().as_str()) {
			let percentage = percent / 100.0;
			let pixels = f64::from(monitor_info.width()) * percentage;
			let pixels = pixels.abs();
			log::error!(
				"percent {} -> percentage {} -> pixels {}",
				percent,
				percentage,
				pixels
			);
			return pixels as i32;
		}
	}
	let pixels_regex = Regex::new(r"(?P<pixels>\d+)(px)?").unwrap();
	if let Some(pixels) = pixels_regex.captures(value.as_ref()) {
		let count = i32::from_str(pixels.name("pixels").unwrap().as_str()).unwrap();
		count
	} else {
		0
	}
}

impl Win32Window {
	pub fn new(hwnd: HWND) -> Self {
		let title = property::get_title(hwnd);
		let process = property::get_process(hwnd);
		let monitor = property::get_monitor(hwnd);
		let rect = property::get_rect(hwnd);
		let origin = rect.origin();
		Win32Window {
			hwnd,
			monitor: monitor,
			window: WindowBuilder::default()
				.title(title)
				.process(process)
				.x(origin.x.to_string())
				.y(origin.y.to_string())
				.w(rect.width().to_string())
				.h(rect.height().to_string())
				.build()
				.unwrap(),
		}
	}

	/// See https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-deferwindowpos
	pub fn update(&self, hdwp: &mut HDWP) {
		let rect = property::get_rect(self.hwnd);
		let origin = rect.origin();
		let mut flags = SWP_NOZORDER | SWP_NOOWNERZORDER | SWP_NOACTIVATE;
		if self.window.x.is_none() && self.window.y.is_none() {
			flags |= SWP_NOMOVE;
		}
		if self.window.w.is_none() && self.window.h.is_none() {
			flags |= SWP_NOSIZE;
		}
		let monitor_info = Win32Monitor::from(self.monitor);
		let monitor_info = Rectangle::from(monitor_info.info.rcWork);
		// TODO match Windows with non-null process and title
		// TODO match Windows with non-null process
		// TODO match Windows with non-null title
		//
		// TODO match Windows with x, y to reposition
		// TODO match Windows with w, h to resize
		// TODO match Windows with maximize to maximize
		// TODO match Windows with maximize_horizontal to maximize_horizontal
		// TODO match Windows with maximize_vertical to maximize_vertical
		let pixels_x = if let Some(x_str) = &self.window.x {
			// Pixels::from_str(&x_str).unwrap().count
			into_pixels(x_str, &monitor_info)
		} else {
			origin.x
		};
		let pixels_y = if let Some(y_str) = &self.window.y {
			// Pixels::from_str(&y_str).unwrap().count
			into_pixels(y_str, &monitor_info)
		} else {
			origin.y
		};
		let pixels_w = if let Some(w_str) = &self.window.w {
			// Pixels::from_str(&w_str).unwrap().count
			into_pixels(w_str, &monitor_info)
		} else {
			rect.width()
		};
		let pixels_h = if let Some(h_str) = &self.window.h {
			// Pixels::from_str(&h_str).unwrap().count
			into_pixels(h_str, &monitor_info)
		} else {
			rect.height()
		};
		if let Some(title) = &self.window.title {
			log::trace!("winapi::DeferWindowPos -- {} for \"{}\"", rect, title);
		} else if let Some(process) = &self.window.process {
			log::trace!("winapi::DeferWindowPos -- {} for \"{}\"", rect, process);
		}
		*hdwp = unsafe {
			DeferWindowPos(
				*hdwp,
				self.hwnd,
				WM_NULL as HWND,
				pixels_x,
				pixels_y,
				pixels_w,
				pixels_h,
				flags,
			)
		};
		if self.window.minimized.is_some() {
			log::trace!("winapi::ShowWindow minimized");
			unsafe {
				ShowWindow(self.hwnd, SW_SHOWMINIMIZED);
			}
		}
		if self.window.maximized.is_some() {
			log::trace!("winapi::ShowWindow maximized");
			unsafe {
				ShowWindow(self.hwnd, SW_SHOWMAXIMIZED);
			}
		}
		if *hdwp == NULL {
			log::error!(
				"winapi::DeferWindowPos error: {}",
				std::io::Error::last_os_error()
			);
		}
	}

	pub fn layout(&mut self, hdwp: &mut HDWP, layout: &Window) {
		self.window = layout.clone();
		self.update(hdwp);
	}

	#[allow(dead_code)]
	pub fn get_rect(&self) -> Rectangle {
		property::get_rect(self.hwnd)
	}

	#[allow(dead_code)]
	pub fn get_title(&self) -> String {
		property::get_title(self.hwnd)
	}

	#[allow(dead_code)]
	pub fn get_process(&self) -> String {
		property::get_process(self.hwnd)
	}

	#[allow(dead_code)]
	pub fn get_monitor(&self) -> HMONITOR {
		property::get_monitor(self.hwnd)
	}
}

impl Default for Win32Window {
	fn default() -> Self {
		Win32Window::new(0 as HWND)
	}
}

impl From<HWND> for Win32Window {
	fn from(hwnd: HWND) -> Self {
		Win32Window::new(hwnd)
	}
}

#[derive(Debug)]
pub struct Win32Provider;

impl Default for Win32Provider {
	fn default() -> Self {
		Win32Provider {}
	}
}

pub struct Win32Monitor {
	hmonitor: HMONITOR,
	info: MONITORINFOEXW,
}

impl Win32Monitor {
	pub fn new(hmonitor: HMONITOR) -> Self {
		let info = unsafe {
			let mut monitor_info: MONITORINFOEXW = mem::zeroed();
			monitor_info.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;
			let monitor_info_ptr = <*mut _>::cast(&mut monitor_info);
			let result = GetMonitorInfoW(hmonitor, monitor_info_ptr);
			if result == TRUE {
				monitor_info
			} else {
				panic!("GetMonitorInfoW Error: {}", std::io::Error::last_os_error())
			}
		};
		let monitor = Win32Monitor { hmonitor, info };
		log::trace!("winapi::GetMonitorInfoW returned -- {}", monitor);
		monitor
	}

	pub fn title(&self) -> String {
		let name = match &self.info.szDevice[..].iter().position(|c| *c == 0) {
			Some(len) => OsString::from_wide(&self.info.szDevice[0..*len]),
			None => OsString::from_wide(&self.info.szDevice[0..self.info.szDevice.len()]),
		};
		name.into_string().unwrap()
	}
}

impl From<HMONITOR> for Win32Monitor {
	fn from(value: HMONITOR) -> Self {
		Win32Monitor::new(value)
	}
}

impl std::fmt::Display for Win32Monitor {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let rc_monitor = Rectangle::from(self.info.rcMonitor);
		let rc_work = Rectangle::from(self.info.rcWork);
		write!(
			f,
			r#"[name: "{}", handle: {}, rcMonitor: {}, rcWork: {}]"#,
			self.title(),
			self.hmonitor as u32,
			rc_monitor,
			rc_work,
		)
		// write!(
		// 	f,
		// 	r#"[name: "{}", handle: {}, left: {}, right: {}, top: {}, bottom: {}]"#,
		// 	name.to_str().unwrap(),
		// 	self.hmonitor as u32,
		// 	self.info.rcWork.left,
		// 	self.info.rcWork.right,
		// 	self.info.rcWork.top,
		// 	self.info.rcWork.bottom
		// )
	}
}

pub fn list_monitors() -> Vec<Win32Monitor> {
	let mut monitors = Vec::new();
	let userdata = &mut monitors as *mut _;
	let result = unsafe {
		EnumDisplayMonitors(
			std::ptr::null_mut(),
			std::ptr::null(),
			Some(monitor_enum_callback),
			userdata as LPARAM,
		)
	};
	if result != TRUE {
		panic!(
			"Could not enumerate monitors: {}",
			std::io::Error::last_os_error()
		)
	}
	monitors
}

unsafe extern "system" fn monitor_enum_callback(
	monitor: HMONITOR,
	_hdc: HDC,
	_rect: LPRECT,
	userdata: LPARAM,
) -> i32 {
	let monitors: &mut Vec<Win32Monitor> = mem::transmute(userdata);
	let monitor = Win32Monitor::from(monitor);
	monitors.push(monitor);
	TRUE
}

pub fn list_windows<'a>() -> Option<Vec<Win32Window>> {
	let mut list = Vec::new();
	let struct_ptr = &mut list as *mut Vec<Win32Window>;
	unsafe {
		EnumWindows(Some(filter_windows_callback), struct_ptr as LPARAM);
	}
	Some(list)
}

unsafe extern "system" fn filter_windows_callback(hwnd: HWND, l_param: LPARAM) -> i32 {
	let window_list = l_param as *mut Vec<Win32Window>;
	match check_valid_window(hwnd) {
		Some(window) => (*window_list).push(window),
		None => {}
	}
	1
}

fn basename(path: &str) -> String {
	let path = Path::new(&path);
	let window_process = match path.file_name() {
		Some(path) => path.to_str().unwrap(),
		None => "",
	};
	window_process.to_owned()
}

fn check_valid_window(hwnd: HWND) -> Option<Win32Window> {
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
		let window_title = property::get_title(hwnd);
		let window_process = property::get_process(hwnd);
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
	}
	if cloaked_value != 0 {
		true
	} else {
		false
	}
}

pub mod property {
	use super::*;

	pub fn get_rect(hwnd: HWND) -> Rectangle {
		let mut rect = Rectangle::default();
		let rect_ptr = &mut rect.0 as *mut RECT;
		let result = unsafe { GetWindowRect(hwnd, rect_ptr) };
		if result == 0 {
			log::warn!(
				"winapi::GetWindowRect error: {}",
				std::io::Error::last_os_error()
			);
		}
		log::trace!("winapi::GetWindowRect({}) returned {}", hwnd as u32, rect,);
		rect
	}

	pub fn get_title(hwnd: HWND) -> String {
		let mut window_title: [WCHAR; MAX_WINDOW_TITLE_LENGTH] = [0; MAX_WINDOW_TITLE_LENGTH];
		let result = unsafe {
			GetWindowTextW(
				hwnd,
				window_title.as_mut_ptr(),
				MAX_WINDOW_TITLE_LENGTH as i32,
			)
		};
		if result == 0 {
			log::warn!(
				"winapi::GetWindowTextW error: {}",
				std::io::Error::last_os_error()
			);
		}
		let mut window_title = window_title.to_vec();
		if let Some(first) = window_title.iter().position(|&b| b == 0) {
			window_title.truncate(first);
		}
		let title = String::from_utf16(&window_title).unwrap();
		log::trace!(
			"winapi::GetWindowTextW({}) returned \"{}\"",
			hwnd as i32,
			title
		);
		title
	}

	pub fn get_process(hwnd: HWND) -> String {
		let mut proc_id: DWORD = 0;
		let mut window_process: [WCHAR; MAX_PATH] = [0; MAX_PATH];
		let result = unsafe {
			GetWindowThreadProcessId(hwnd, &mut proc_id);
			let process_handle: HANDLE =
				OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, proc_id);
			let result = GetModuleFileNameExW(
				process_handle,
				NULL as HINSTANCE,
				window_process.as_mut_ptr(),
				MAX_PATH as u32,
			);
			CloseHandle(process_handle);
			result
		};
		if result == 0 {
			log::warn!(
				"winapi::GetModuleFileNameExW error: {}",
				std::io::Error::last_os_error()
			);
		}
		let mut window_process = window_process.to_vec();
		if let Some(first) = window_process.iter().position(|&b| b == 0) {
			window_process.truncate(first)
		}
		let process = String::from_utf16(&window_process).unwrap();
		log::trace!(
			"winapi::GetWindowThreadProcessId({}) returned \"{}\"",
			hwnd as i32,
			process
		);
		process
	}

	pub fn get_monitor(hwnd: HWND) -> HMONITOR {
		#[allow(unused_assignments)]
		let monitor_handle = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY) };
		log::trace!(
			"winapi::MonitorFromWindow({}) returned {}",
			hwnd as i32,
			monitor_handle as i32
		);
		monitor_handle
	}
}

fn find_match_by_title_and_process(
	windows: &Vec<Win32Window>,
	title_regex: &Regex,
	process_regex: &Regex,
) -> Option<PlatformWindow> {
	for w in windows {
		match (&w.window.title, &w.window.process) {
			(Some(title), Some(process)) => {
				if title_regex.is_match(&title) && process_regex.is_match(&process) {
					return Some(w.clone());
				}
			}
			_ => {}
		}
	}
	None
}

fn find_match_by_title(windows: &Vec<Win32Window>, title_regex: &Regex) -> Option<PlatformWindow> {
	for w in windows {
		match &w.window.title {
			Some(title) => {
				if title_regex.is_match(&title) {
					return Some(w.clone());
				}
			}
			_ => {}
		}
	}
	None
}

fn find_match_by_process(
	windows: &Vec<Win32Window>,
	process_regex: &Regex,
) -> Option<PlatformWindow> {
	for w in windows {
		match &w.window.process {
			Some(process) => {
				if process_regex.is_match(&process) {
					return Some(w.clone());
				}
			}
			_ => {}
		}
	}
	None
}

fn find_match(windows: &Vec<Win32Window>, win: &Window) -> Option<PlatformWindow> {
	match (&win.title, &win.process) {
		(Some(title_regex), Some(process_regex)) => {
			match (Regex::new(title_regex), Regex::new(process_regex)) {
				(Ok(title_regex), Ok(process_regex)) => {
					return find_match_by_title_and_process(windows, &title_regex, &process_regex);
				}
				_ => {}
			}
		}
		(Some(title_regex), None) => {
			if let Ok(title_regex) = Regex::new(title_regex) {
				return find_match_by_title(windows, &title_regex);
			}
		}
		(None, Some(process_regex)) => {
			if let Ok(process_regex) = Regex::new(process_regex) {
				return find_match_by_process(windows, &process_regex);
			}
		}
		_ => {}
	}
	None
}

impl WindowProvider for Win32Provider {
	fn screens(&self) -> Vec<Screen> {
		let mut screen_map = HashMap::new();
		let mut screen_count = 0;
		for win32monitor in list_monitors() {
			let screen = ScreenBuilder::default().id(screen_count).build().unwrap();
			screen_map.insert(win32monitor.hmonitor, screen);
			log::debug!("Screen {}", win32monitor.hmonitor as i32);
			screen_count += 1;
		}

		if let Some(windows) = list_windows() {
			for window in windows {
				log::debug!("Window {}", window);
				if let Some(screen) = screen_map.get_mut(&window.monitor) {
					log::debug!("HERE {}", screen);
					let window = WindowBuilder::default()
						.title(window.window.title)
						.process(window.window.process)
						.x(window.window.x)
						.y(window.window.y)
						.w(window.window.w)
						.h(window.window.h)
						.build()
						.unwrap();
					screen.windows.push(window);
				}
			}
		}
		screen_map.values().cloned().collect()
	}

	fn layout(&self, layout: &Layout) {
		let windows = list_windows().or(Some(Vec::new())).unwrap();
		#[allow(unused_assignments)]
		let mut hdwp = NULL;
		unsafe {
			hdwp = BeginDeferWindowPos(1);
		}

		for s in &layout.screens {
			for layout_window in &s.windows {
				if let Some(mut win32window) = find_match(&windows, &layout_window) {
					win32window.layout(&mut hdwp, &layout_window);
				}
			}
		}

		if hdwp != NULL {
			unsafe {
				EndDeferWindowPos(hdwp);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	mod rectangle {
		use super::super::*;

		#[test]
		fn calc_width() {
			let mut r = Rectangle::from(Point::new(0, 0));
			r.0.right = 2;
			assert_eq!(r.width(), 2);
		}

		#[test]
		fn calc_height() {
			let mut r = Rectangle::from(Point::new(0, 0));
			r.0.bottom = 3;
			assert_eq!(r.height(), 3);
		}
	}
	mod win32window {
		use super::super::*;

		#[test]
		fn layout_updates_x() {
			let mut hdwp = NULL;
			let w1 = WindowBuilder::default()
				.title(Some("Window 1".to_string()))
				.x(Some("100".to_string()))
				.build()
				.unwrap();
			let mut ws1 = Win32Window::default();
			ws1.window.title = Some("Window 1".to_string());
			ws1.window.x = Some("0".to_string());
			ws1.layout(&mut hdwp, &w1);
			assert_eq!("100", ws1.window.x.unwrap());
		}

		#[test]
		fn layout_updates_y() {
			let mut hdwp = NULL;
			let w1 = WindowBuilder::default()
				.title(Some("Window 1".to_string()))
				.y(Some("100".to_string()))
				.build()
				.unwrap();
			let mut ws1 = Win32Window::default();
			ws1.window.title = Some("Window 1".to_string());
			ws1.window.y = Some("0".to_string());
			ws1.layout(&mut hdwp, &w1);
			assert_eq!("100", ws1.window.y.unwrap());
		}

		#[test]
		fn layout_updates_w() {
			let mut hdwp = NULL;
			let w1 = WindowBuilder::default()
				.title(Some("Window 1".to_string()))
				.w(Some("100".to_string()))
				.build()
				.unwrap();
			let mut ws1 = Win32Window::default();
			ws1.window.title = Some("Window 1".to_string());
			ws1.window.w = Some("0".to_string());
			ws1.layout(&mut hdwp, &w1);
			assert_eq!("100", ws1.window.w.unwrap());
		}

		#[test]
		fn layout_updates_h() {
			let mut hdwp = NULL;
			let w1 = WindowBuilder::default()
				.title(Some("Window 1".to_string()))
				.h(Some("100".to_string()))
				.build()
				.unwrap();
			let mut ws1 = Win32Window::default();
			ws1.window.title = Some("Window 1".to_string());
			ws1.window.h = Some("0".to_string());
			ws1.layout(&mut hdwp, &w1);
			assert_eq!("100", ws1.window.h.unwrap());
		}
	}
}

use std::ptr;
use winapi::{
	shared::{
		minwindef::{BOOL, LPARAM, TRUE},
		windef::{HDC, HMONITOR, LPRECT},
	},
	um::winuser,
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

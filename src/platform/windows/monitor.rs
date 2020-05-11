use crate::{
	error::Result,
	monitor::Monitor,
	platform::windows::{get_dimensions, get_position},
};
use std::{io, mem, ptr};
use winapi::{
	shared::{
		minwindef::{BOOL, DWORD, LPARAM, TRUE},
		windef::{HDC, HMONITOR, LPRECT},
	},
	um::winuser::{self, MONITORINFO, MONITORINFOEXW},
};

#[derive(Debug)]
struct MonitorHandle(HMONITOR);

impl From<MonitorHandle> for Monitor {
	fn from(hmonitor: MonitorHandle) -> Self {
		let monitor_info = get_monitor_info_ex(hmonitor);
		let mut monitor = Monitor::new();
		if let Ok(monitor_info) = monitor_info {
			monitor.position = get_position(monitor_info.rcMonitor);
			monitor.size = get_dimensions(monitor_info.rcMonitor);
			if let Some(first_null) = monitor_info.szDevice.iter().position(|&b| b == 0) {
				monitor.name = String::from_utf16_lossy(&monitor_info.szDevice);
				monitor.name.truncate(first_null);
			}
		}
		monitor
	}
}

unsafe extern "system" fn monitor_enum_proc(
	hmonitor: HMONITOR,
	_hdc: HDC,
	_place: LPRECT,
	data: LPARAM,
) -> BOOL {
	let monitors = data as *mut Vec<Monitor>;
	(*monitors).push(MonitorHandle(hmonitor).into());
	TRUE // continue enumeration
}

pub fn list_monitors() -> Result<Vec<Monitor>> {
	let mut monitors: Vec<Monitor> = Vec::new();
	let status = unsafe {
		winuser::EnumDisplayMonitors(
			ptr::null_mut(),
			ptr::null_mut(),
			Some(monitor_enum_proc),
			&mut monitors as *mut _ as LPARAM,
		)
	};
	if status == 0 {
		Err(io::Error::last_os_error().into())
	} else {
		Ok(monitors)
	}
}

fn get_monitor_info_ex(handle: MonitorHandle) -> Result<winuser::MONITORINFOEXW> {
	let mut monitor_info: MONITORINFOEXW = unsafe { mem::zeroed() };
	monitor_info.cbSize = mem::size_of::<MONITORINFOEXW>() as DWORD;
	let status = unsafe {
		winuser::GetMonitorInfoW(
			handle.0,
			&mut monitor_info as *mut MONITORINFOEXW as *mut MONITORINFO,
		)
	};
	if status == 0 {
		Err(io::Error::last_os_error().into())
	} else {
		Ok(monitor_info)
	}
}

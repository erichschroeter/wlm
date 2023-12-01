use std::{
	ffi::{c_ulong, CStr},
	ptr,
};

use x11::xlib::{self, XInternAtom};

use crate::{
	layout::{Screen, Window, WindowBuilder},
	Dimensions, Point, WindowProvider,
};
// extern crate x11;

// use std::ptr;
// use x11::xlib;

#[derive(Debug)]
pub struct X11Provider;

impl Default for X11Provider {
	fn default() -> Self {
		X11Provider {}
	}
}

impl WindowProvider for X11Provider {
	fn screens(&self) -> Vec<crate::layout::Screen> {
		let windows = list_windows();
		let mut screen = Screen::new();
		screen.windows = windows.iter().map(|x11w| x11w.window.clone()).collect();
		let screens = vec![screen];
		screens
	}

	fn layout(&self, _config: &crate::layout::Layout) {}
}

#[derive(Debug, Clone)]
pub struct X11Screen {
	pub display_ptr: *mut xlib::Display,
	pub id: i32,
	pub name: String,
	pub dimensions: Dimensions,
}

impl Default for X11Screen {
	fn default() -> Self {
		let display_ptr = unsafe { xlib::XOpenDisplay(ptr::null()) };
		if display_ptr.is_null() {
			panic!("XOpenDisplay failed to get default screen");
		}
		let display_name_ptr = unsafe { CStr::from_ptr(xlib::XDisplayName(ptr::null())) };
		let display_name = display_name_ptr
			.to_str()
			.to_owned()
			.unwrap_or("unknown")
			.to_string();
		let id = unsafe { xlib::XDefaultScreen(display_ptr) };
		let width = unsafe { xlib::XDisplayWidth(display_ptr, id) };
		let height = unsafe { xlib::XDisplayHeight(display_ptr, id) };
		X11Screen {
			display_ptr,
			id,
			name: display_name,
			dimensions: Dimensions::new(width, height),
		}
	}
}

impl std::fmt::Display for X11Screen {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"id: {}, name: \"{}\", dimensions: {}",
			self.display_ptr as u32, self.name, self.dimensions
		)
	}
}

#[derive(Debug, Clone)]
pub struct X11Window {
	pub id: u64,
	pub window: Window,
}

impl X11Window {
	pub fn new(id: u64) -> Self {
		X11Window {
			id,
			window: WindowBuilder::default().build().unwrap(),
		}
	}

	pub fn populate(mut self, screen: &X11Screen) -> Self {
		let size = self.request_size(screen);
		self.window.w = Some(size.width.to_string());
		self.window.h = Some(size.height.to_string());
		let location = self.request_location(screen);
		self.window.x = Some(location.x.to_string());
		self.window.y = Some(location.y.to_string());
		self.window.title = self.request_name(screen).into();
		self
	}

	pub fn request_size(&self, screen: &X11Screen) -> Dimensions {
		let mut attributes: xlib::XWindowAttributes = unsafe { std::mem::zeroed() };
		unsafe {
			xlib::XGetWindowAttributes(screen.display_ptr, self.id, &mut attributes);
		}
		Dimensions::new(attributes.width, attributes.height)
	}

	pub fn request_location(&self, screen: &X11Screen) -> Point {
		// The coordinates in attr are relative to the parent window.  If
		// the parent window is the root window, then the coordinates are
		// correct.  If the parent window isn't the root window --- which
		// is likely --- then we translate them.
		let mut attributes: xlib::XWindowAttributes = unsafe { std::mem::zeroed() };
		unsafe {
			xlib::XGetWindowAttributes(screen.display_ptr, self.id, &mut attributes);
		}

		let mut returned_root = 0;
		let mut parent = 0;
		let mut children = ptr::null_mut();
		let mut children_count = 0;

		unsafe {
			xlib::XQueryTree(
				screen.display_ptr,
				self.id as c_ulong,
				&mut returned_root,
				&mut parent,
				&mut children,
				&mut children_count,
			);
			if !children.is_null() {
				xlib::XFree(children as *mut _);
			}
		}

		if parent == attributes.root {
			Point::new(attributes.x, attributes.y)
		} else {
			let mut x = 0;
			let mut y = 0;
			let unused_child = ptr::null_mut();
			unsafe {
				xlib::XTranslateCoordinates(
					screen.display_ptr,
					self.id,
					attributes.root,
					0,
					0,
					&mut x,
					&mut y,
					unused_child,
				);
			}
			Point::new(x, y)
		}
	}

	fn request_window_property_by_atom(&self, screen: &X11Screen, atom: u64) -> Option<String> {
		let mut actual_type_return = 0;
		let mut actual_format_return = 0;
		let mut nitems_return = 0;
		let mut bytes_after_return = 0;
		let mut prop_return = std::ptr::null_mut();
		// let name = xlib::XGetWindowProperty(screen.display_ptr, self.id, atom_net_wm_name, 0, 0, xlib::False, 6, 5, 4, 3, 2, 1)
		let status = unsafe {
			xlib::XGetWindowProperty(
				screen.display_ptr,
				self.id,
				atom,
				0,
				std::mem::size_of::<u64>() as i64,
				xlib::False,
				xlib::AnyPropertyType as u64,
				&mut actual_type_return,
				&mut actual_format_return,
				&mut nitems_return,
				&mut bytes_after_return,
				&mut prop_return,
			)
		};
		if status == xlib::Success.into() {
			if !prop_return.is_null() {
				let window_name = unsafe { CStr::from_ptr(prop_return as *const i8) }
					.to_string_lossy()
					.into_owned();
				unsafe {
					xlib::XFree(prop_return as *mut std::ffi::c_void);
				}
				return Some(window_name);
			}
		} else {
			log::warn!("xlib::XGetWindowProperty failed!");
		}
		None
	}

	pub fn request_name(&self, screen: &X11Screen) -> String {
		let atom_net_wm_name = unsafe {
			let atom_name = "_NET_WM_NAME\0";
			XInternAtom(
				screen.display_ptr,
				atom_name.as_ptr() as *const i8,
				xlib::False,
			)
		};
		let atom_wm_name = unsafe {
			let atom_name = "WM_NAME\0";
			XInternAtom(
				screen.display_ptr,
				atom_name.as_ptr() as *const i8,
				xlib::False,
			)
		};

		if let Some(name) = self.request_window_property_by_atom(screen, atom_net_wm_name) {
			name
		} else if let Some(name) = self.request_window_property_by_atom(screen, atom_wm_name) {
			name
		} else {
			"".to_string()
		}
	}
}

impl Default for X11Window {
	fn default() -> Self {
		X11Window::new(0)
	}
}

impl std::fmt::Display for X11Window {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// let x: i32 = self.window.x.unwrap_or("0".to_string()).parse().unwrap();
		// let y: i32 = self.window.y.unwrap_or("0".to_string()).parse().unwrap();
		// let location = Point::new(x, y);
		// let w: i32 = self.window.w.unwrap_or("0".to_string()).parse().unwrap();
		// let h: i32 = self.window.h.unwrap_or("0".to_string()).parse().unwrap();
		// let dimensions = Dimensions::new(w, h);
		// write!(f, "id: {}, coordinates: {}, dimensions: {}", self.id, location, dimensions)
		write!(f, "{:#?}", &self)
	}
}

/// https://github.com/jordansissel/xdotool
fn list_windows() -> Vec<X11Window> {
	let mut x11windows = Vec::new();
	let screen = X11Screen::default();
	log::error!("Primary screen: {}", screen);

	let root = unsafe { xlib::XRootWindow(screen.display_ptr, screen.id) };
	let mut returned_root = 0;
	let mut returned_parent = 0;
	let mut children = ptr::null_mut();
	let mut num_children = 0;

	unsafe {
		xlib::XQueryTree(
			screen.display_ptr,
			root,
			&mut returned_root,
			&mut returned_parent,
			&mut children,
			&mut num_children,
		);
	}

	for i in 0..num_children {
		let window_index = unsafe { *children.offset(i as isize) };
		let window = X11Window::new(window_index).populate(&screen);
		x11windows.push(window);
	}
	// unsafe {
	//     let display = xlib::XOpenDisplay(ptr::null());
	//     if display.is_null() {
	//         eprintln!("Cannot open display");
	//         std::process::exit(1);
	//     }

	// 	// TODO https://unix.stackexchange.com/questions/573121/get-current-screen-dimensions-via-xlib-using-c
	//     let screen = xlib::XDefaultScreen(display);
	//     let root = xlib::XRootWindow(display, screen);

	//     let mut returned_root = 0;
	//     let mut returned_parent = 0;
	//     let mut top_level_windows = ptr::null_mut();
	//     let mut num_top_level_windows = 0;

	//     xlib::XQueryTree(display, root, &mut returned_root, &mut returned_parent, &mut top_level_windows, &mut num_top_level_windows);

	//     for i in 0..num_top_level_windows {
	//         let window_index = *top_level_windows.offset(i as isize);
	// 		let mut window = WindowBuilder::default();

	//         let mut attributes: xlib::XWindowAttributes = std::mem::zeroed();
	//         xlib::XGetWindowAttributes(display, window_index, &mut attributes);

	//         if attributes.map_state == xlib::IsViewable {
	//             let mut name = ptr::null_mut();
	//             xlib::XFetchName(display, window_index, &mut name);
	//             if !name.is_null() {
	//                 let window_name = std::ffi::CStr::from_ptr(name).to_string_lossy();
	//                 println!("Window ID: {}, Name: {}", window_index, window_name);
	// 				window.title(window_name.to_string());
	//                 xlib::XFree(name as *mut _);
	//             }

	// 			window.x(Some(i32::to_string(&attributes.x)));
	// 			window.y(Some(i32::to_string(&attributes.y)));
	// 			window.w(Some(i32::to_string(&attributes.width)));
	// 			window.h(Some(i32::to_string(&attributes.height)));

	// 			// Define the atom for _NET_WM_PID
	// 			let net_wm_pid_atom = {
	// 				let atom_name = "_NET_WM_PID\0";
	// 				xlib::XInternAtom(display, atom_name.as_ptr() as *const i8, xlib::False)
	// 			};

	// 			let mut actual_type_return = 0;
	// 			let mut actual_format_return = 0;
	// 			let mut nitems_return = 0;
	// 			let mut bytes_after_return = 0;
	// 			let mut prop_return = std::ptr::null_mut();

	// 			// Get the _NET_WM_PID property
	// 			if xlib::XGetWindowProperty(
	// 					display,
	// 					window_index,
	// 					net_wm_pid_atom,
	// 					0,
	// 					std::mem::size_of::<u64>() as i64,
	// 					xlib::False,
	// 					xlib::AnyPropertyType as u64,
	// 					&mut actual_type_return,
	// 					&mut actual_format_return,
	// 					&mut nitems_return,
	// 					&mut bytes_after_return,
	// 					&mut prop_return,
	// 				) == xlib::Success.into()
	// 			{
	// 				if !prop_return.is_null() && actual_format_return == 32 {
	// 					let pid_ptr = prop_return as *const u64;
	// 					let pid = *pid_ptr;
	// 					println!("Window ID: {}, PID: {}", window_index, pid);

	// 					xlib::XFree(prop_return as *mut std::ffi::c_void);
	// 					let proc_path = format!("/proc/{}/comm", pid);
	// 					match std::fs::read_to_string(proc_path) {
	// 						Ok(process_name) => {
	// 							let process_name = process_name.trim(); // Remove any trailing newline
	// 							window.process(Some(process_name.to_string()));
	// 							log::debug!("Window ID: {}, Process Name: {}", window_index, process_name);
	// 						}
	// 						Err(e) => {
	// 							log::warn!("Failed to read process name for PID {}: {}", pid, e);
	// 						}
	// 					}
	// 				}
	// 			} else {
	// 				log::warn!("No _NET_WM_PID property for Window ID: {}", window_index);
	// 			}
	//         }
	// 		let x11window = X11Window {
	// 			window: window.build().unwrap(),
	// 		};
	// 		x11windows.push(x11window);
	//     }

	//     if !top_level_windows.is_null() {
	//         xlib::XFree(top_level_windows as *mut _);
	//     }

	//     xlib::XCloseDisplay(display);
	// }
	x11windows
}

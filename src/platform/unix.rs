use std::{ptr, ffi::CStr};

use x11::xlib;

use crate::{WindowProvider, layout::{Window, Screen, WindowBuilder}, Dimensions};
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
		for w in windows {
			let window = WindowBuilder::default()
				.title(w.window.title)
				.build().unwrap();
			screen.windows.push(window);
		}
		let screens = vec![screen];
		screens
	}

	fn layout(&self, _config: &crate::layout::Layout) {}
}

#[derive(Debug, Clone)]
pub struct X11Screen {
	pub display_ptr: *mut xlib::Display,
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
		let display_name = display_name_ptr.to_str().to_owned().unwrap_or("unknown").to_string();
		let screen_num = unsafe { xlib::XDefaultScreen(display_ptr) };
		let width = unsafe { xlib::XDisplayWidth(display_ptr, screen_num) };
		let height = unsafe { xlib::XDisplayHeight(display_ptr, screen_num) };
		X11Screen { display_ptr, name: display_name, dimensions: Dimensions::new(width, height) }
	}
}

impl std::fmt::Display for X11Screen {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "id: {}, name: \"{}\", dimensions: {}", self.display_ptr as u32, self.name, self.dimensions)
	}
}

#[derive(Debug, Clone)]
pub struct X11Window {
	pub id: i32,
	pub window: Window,
}

impl X11Window {
	pub fn new(id: i32) -> Self {
		X11Window { id, window: WindowBuilder::default().build().unwrap() }
	}
}

impl Default for X11Window {
	fn default() -> Self {
		X11Window::new(0)
	}
}

impl std::fmt::Display for X11Window {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "id: {}", self.id)
	}
}

// mod xdo {
//     use crate::Point;

//     use super::{X11Screen, X11Window};

// 	pub fn get_window_location(screen: &X11Screen, window: &X11Window) -> Point {
//         let mut returned_root = 0;
//         let mut returned_parent = 0;
//         let mut top_level_windows = ptr::null_mut();
//         let mut num_top_level_windows = 0;

//         xlib::XQueryTree(screen., window.id, &mut returned_root, &mut returned_parent, &mut top_level_windows, &mut num_top_level_windows);

// 		Point::new(0, 0)
// 	}
// }

/// https://github.com/jordansissel/xdotool
fn list_windows() -> Vec<X11Window> {
	let mut x11windows = Vec::new();
	let screen = X11Screen::default();
	log::error!("Primary screen: {}", screen);
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

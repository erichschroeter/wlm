use std::ptr;

use x11::xlib;

use crate::{WindowProvider, layout::{Window, Screen, WindowBuilder}};
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
struct X11Window {
	pub window: Window,
}

fn list_windows() -> Vec<X11Window> {
	let mut x11windows = Vec::new();
    unsafe {
        let display = xlib::XOpenDisplay(ptr::null());
        if display.is_null() {
            eprintln!("Cannot open display");
            std::process::exit(1);
        }

        let screen = xlib::XDefaultScreen(display);
        let root = xlib::XRootWindow(display, screen);

        let mut returned_root = 0;
        let mut returned_parent = 0;
        let mut top_level_windows = ptr::null_mut();
        let mut num_top_level_windows = 0;

        xlib::XQueryTree(display, root, &mut returned_root, &mut returned_parent, &mut top_level_windows, &mut num_top_level_windows);

        for i in 0..num_top_level_windows {
            let window_index = *top_level_windows.offset(i as isize);
			let mut window = WindowBuilder::default();

            let mut attributes: xlib::XWindowAttributes = std::mem::zeroed();
            xlib::XGetWindowAttributes(display, window_index, &mut attributes);

            if attributes.map_state == xlib::IsViewable {
                let mut name = ptr::null_mut();
                xlib::XFetchName(display, window_index, &mut name);
                if !name.is_null() {
                    let window_name = std::ffi::CStr::from_ptr(name).to_string_lossy();
                    println!("Window ID: {}, Name: {}", window_index, window_name);
					window.title(window_name.to_string());
                    xlib::XFree(name as *mut _);
                }

				window.x(Some(i32::to_string(&attributes.x)));
				window.y(Some(i32::to_string(&attributes.y)));
				window.w(Some(i32::to_string(&attributes.width)));
				window.h(Some(i32::to_string(&attributes.height)));
            }
			let x11window = X11Window {
				window: window.build().unwrap(),
			};
			x11windows.push(x11window);
        }

        if !top_level_windows.is_null() {
            xlib::XFree(top_level_windows as *mut _);
        }

        xlib::XCloseDisplay(display);
    }
	x11windows
}

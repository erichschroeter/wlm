use crate::config::Config;
use crate::platform as sys;

pub struct WindowManager<'a> {
	config: Option<&'a Config>,
}

impl<'a> WindowManager<'a> {
	pub fn new(config: Option<&'a Config>) -> Self {
		WindowManager { config }
	}

	pub fn monitors(&self) -> Vec<sys::Monitor> {
		sys::list_monitors()
	}

	pub fn windows(&self) -> Option<Vec<sys::WindowState>> {
		sys::list_windows(self.config)
	}

	pub fn layout(&self) {
		sys::layout_windows(self.config)
	}

	pub fn print(&self) {
		sys::print_windows(self.config)
	}
}

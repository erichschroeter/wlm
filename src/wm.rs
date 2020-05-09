use crate::config::Config;
use crate::platform;

pub struct WindowManager<'a> {
	config: Option<&'a Config>,
}

impl<'a> WindowManager<'a> {
	pub fn new(config: Option<&'a Config>) -> Self {
		WindowManager { config }
	}

	pub fn monitors(&self) -> Vec<platform::Monitor> {
		platform::list_monitors()
	}

	pub fn windows(&self) -> Option<Vec<platform::WindowState>> {
		platform::list_windows(self.config)
	}

	pub fn layout(&self) {
		platform::layout_windows(self.config)
	}

	pub fn print(&self) {
		platform::print_windows(self.config)
	}
}

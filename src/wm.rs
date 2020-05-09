use crate::{
	config::{Config, Window},
	platform,
};

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

	pub fn windows(config: &Config) -> Option<Vec<Window>> {
		platform::list_windows(Some(config))
	}

	pub fn layout(&self) {
		platform::layout_windows(self.config)
	}
}

use crate::{
	config::{Config, Window},
	error::Result,
	monitor::Monitor,
	platform,
};

pub struct WindowManager;

impl WindowManager {
	pub fn monitors() -> Vec<Monitor> {
		platform::list_monitors()
	}

	pub fn windows(config: Option<&Config>) -> Result<Vec<Window>> {
		platform::list_windows(config)
	}

	pub fn layout(config: &Config) {
		platform::layout_windows(Some(config))
	}
}

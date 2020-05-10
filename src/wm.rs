use crate::{
	config::{Config, Window},
	monitor::Monitor,
	platform,
};

pub struct WindowManager;

impl WindowManager {
	pub fn monitors() -> Vec<Monitor> {
		platform::list_monitors()
	}

	pub fn windows(config: &Config) -> Option<Vec<Window>> {
		platform::list_windows(Some(config))
	}

	pub fn layout(config: &Config) {
		platform::layout_windows(Some(config))
	}
}

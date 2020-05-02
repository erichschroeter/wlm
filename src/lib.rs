extern crate failure;
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate failure_derive;
extern crate regex;

#[cfg(test)]
use assert_fs::prelude::*;
#[cfg(test)]
use predicates::prelude::*;

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::path::PathBuf;

#[cfg(windows)]
#[path = "platform/mod.rs"]
mod platform;
#[cfg(windows)]
use platform as sys;

pub const MAX_WINDOW_TITLE_LENGTH: usize = 128;

type Result<T> = std::result::Result<T, Error>;

#[derive(Fail, Debug)]
pub enum Error {
	#[fail(display = "IO error: {}", error)]
	Io { error: std::io::Error },
	#[fail(display = "Validation error: {}", error)]
	Validation { error: serde_json::Error },
	#[fail(display = "Invalid property")]
	InvalidProperty,
	#[fail(display = "Invalid index")]
	InvalidIndex,
}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Error {
		Error::Io { error: err }
	}
}

impl From<serde_json::Error> for Error {
	fn from(err: serde_json::Error) -> Error {
		Error::Validation { error: err }
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Builder)]
#[builder(setter(into))]
pub struct Window {
	#[builder(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title: Option<String>,
	#[builder(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub process: Option<String>,
	#[builder(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub x: Option<i32>,
	#[builder(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub y: Option<i32>,
	#[builder(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub w: Option<i32>,
	#[builder(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub h: Option<i32>,
}

impl Window {
	pub fn new() -> Self {
		Window {
			title: None,
			process: None,
			x: None,
			y: None,
			w: None,
			h: None,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Builder)]
#[builder(setter(into))]
pub struct Config {
	#[builder(default)]
	#[serde(skip_serializing, skip_deserializing)]
	pub path: Option<PathBuf>,
	#[builder(default)]
	pub windows: Vec<Window>,
}

impl Config {
	pub fn new() -> Self {
		Config::default()
	}

	pub fn load(path: &str) -> Result<Self> {
		let json = std::fs::read_to_string(path)?;
		let mut config: Config = serde_json::from_str(&json)?;
		config.path = Some(std::path::Path::new(path).to_path_buf());
		Ok(config)
	}

	pub fn save_force(&self, path: &str) -> Result<()> {
		let file = OpenOptions::new()
			.write(true)
			.create(true)
			.truncate(true)
			.open(path)?;
		self.write(file)
	}

	pub fn save(&self, path: &str) -> Result<()> {
		let file = OpenOptions::new().write(true).create(true).open(path)?;
		self.write(file)
	}

	pub fn write<W: std::io::Write>(&self, writer: W) -> Result<()> {
		let _writer = serde_json::to_writer_pretty(writer, &self)?;
		Ok(())
	}

	pub fn search(&self, platform_window: &sys::WindowState) -> Option<Window> {
		for cfg_window in &self.windows {
			match (&cfg_window.title, &cfg_window.process) {
				(Some(cfg_title), Some(cfg_process)) => {
					match (Regex::new(cfg_title), Regex::new(cfg_process)) {
						(Ok(title_re), Ok(process_re)) => {
							match (&platform_window.title, &platform_window.process) {
								(Some(plat_title), Some(plat_process)) => {
									if title_re.is_match(plat_title)
										&& process_re.is_match(plat_process)
									{
										return Some(cfg_window.clone());
									}
								}
								_ => {}
							}
						}
						_ => {}
					}
				}
				(Some(cfg_title), None) => {
					if let Ok(re) = Regex::new(cfg_title) {
						if let Some(plat_title) = &platform_window.title {
							if re.is_match(plat_title) {
								return Some(cfg_window.clone());
							}
						}
					}
				}
				(None, Some(cfg_process)) => {
					if let Ok(re) = Regex::new(cfg_process) {
						if let Some(plat_process) = &platform_window.process {
							if re.is_match(plat_process) {
								return Some(cfg_window.clone());
							}
						}
					}
				}
				_ => {}
			}
		}
		None
	}

	pub fn parse_property_string(the_string: &str) -> Result<(usize, String)> {
		let tokens: Vec<&str> = the_string.split(".").collect();
		match tokens.len() {
			3 => match tokens[0] {
				"windows" => {
					if let Ok(index) = tokens[1].parse::<usize>() {
						match tokens[2] {
							"x" | "y" | "w" | "h" => return Ok((index, tokens[2].to_string())),
							_ => {}
						}
					} else {
						return Err(Error::InvalidIndex);
					}
				}
				_ => {}
			},
			_ => {}
		}
		Err(Error::InvalidProperty)
	}

	pub fn window_at(&mut self, index: usize) -> Result<&mut Window> {
		match self.windows.get_mut(index) {
			Some(window) => Ok(window),
			None => Err(Error::InvalidIndex),
		}
	}
}

impl Default for Config {
	fn default() -> Self {
		Self {
			path: None,
			windows: Vec::new(),
		}
	}
}

pub struct WindowManager<'a> {
	config: Option<&'a Config>,
}

pub trait ProvidesWindowList<'a> {
	fn windows(&self, config: Option<&'a Config>) -> Option<Vec<sys::WindowState>>;
}

impl<'a> WindowManager<'a> {
	pub fn new(config: Option<&'a Config>) -> Self {
		WindowManager { config }
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

fn shrink(the_string: &str, shrink_len: usize) -> String {
	if the_string.len() > shrink_len {
		let mut shrinked = String::new();
		if shrink_len % 2 == 0 {
			shrinked.push_str(&the_string[..(shrink_len / 2 - 2)]);
		} else {
			shrinked.push_str(&the_string[..(shrink_len / 2 - 1)]);
		}
		shrinked.push_str("...");
		shrinked.push_str(&the_string[(the_string.len() - (shrink_len / 2) + 1)..]);
		shrinked
	} else {
		the_string.to_string()
	}
}

pub trait TablePrinter {
	fn print_header<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()>;
	fn print_table<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()>;
}

impl TablePrinter for Config {
	fn print_header<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
		let mut buffer = String::new();
		buffer.push_str(&format!("{:^20}", "Title"));
		buffer.push_str(&format!(" | {:^20}", "Process"));
		buffer.push_str(&format!(" |{:^6}", "X"));
		buffer.push_str(&format!(" |{:^6}", "Y"));
		buffer.push_str(&format!(" |{:^6}", "W"));
		buffer.push_str(&format!(" |{:^6}", "H"));
		write!(w, "{}\n", buffer.to_string())
	}

	fn print_table<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
		let mut buffer = String::new();
		for window in &self.windows {
			if let Some(title) = &window.title {
				buffer.push_str(&format!("{:>20}", shrink(title, 20)));
			} else {
				buffer.push_str("                    ");
			}
			if let Some(process) = &window.process {
				buffer.push_str(&format!(" | {:>20}", shrink(process, 20)));
			} else {
				buffer.push_str(" |                     ");
			}
			if let Some(x) = &window.x {
				buffer.push_str(&format!(" |{:>6}", x.to_string()));
			} else {
				buffer.push_str(" |      ");
			}
			if let Some(y) = &window.y {
				buffer.push_str(&format!(" |{:>6}", y.to_string()));
			} else {
				buffer.push_str(" |      ");
			}
			if let Some(w) = &window.w {
				buffer.push_str(&format!(" |{:>6}", w.to_string()));
			} else {
				buffer.push_str(" |      ");
			}
			if let Some(h) = &window.h {
				buffer.push_str(&format!(" |{:>6}", h.to_string()));
			} else {
				buffer.push_str(" |      ");
			}
			buffer.push_str("\n");
		}
		write!(w, "{}", buffer.to_string())
	}
}

#[cfg(test)]
mod tests {
	mod parse_property_string {
		use super::super::*;

		#[test]
		fn windows_0_x() {
			assert_eq!(
				(0, "x".to_string()),
				Config::parse_property_string("windows.0.x").unwrap()
			);
		}
	}

	mod print_table {
		use super::super::*;

		#[test]
		fn config_with_single_window() {
			let mut config = Config::new();
			let window_1 = WindowBuilder::default()
				.title(Some(String::from("Window 1")))
				.process(Some(String::from("example.exe")))
				.x(Some(100))
				.y(Some(100))
				.w(Some(100))
				.h(Some(100))
				.build()
				.unwrap();
			config.windows.push(window_1);
			let mut buffer = Vec::new();
			assert!(config.print_table(&mut buffer).is_ok());
			assert_eq!(
				"            Window 1 |          example.exe |   100 |   100 |   100 |   100\n",
				std::str::from_utf8(&buffer).unwrap()
			);
		}
	}

	mod shrink {
		use super::super::*;

		#[test]
		fn prefix_smaller_than_suffix_when_given_even_length_string_and_even_shrink_length() {
			assert_eq!("112...9900", shrink("11223344556677889900", 10));
		}

		#[test]
		fn prefix_and_suffix_same_length_when_given_even_length_string_and_odd_shrink_length() {
			assert_eq!("112...900", shrink("11223344556677889900", 9));
		}

		#[test]
		fn prefix_and_suffix_same_length_when_given_odd_length_string_and_even_shrink_length() {
			assert_eq!("112...8990", shrink("1122334455667788990", 10));
		}

		#[test]
		fn prefix_and_suffix_same_length_when_given_odd_length_string_and_odd_shrink_length() {
			assert_eq!("112...990", shrink("1122334455667788990", 9));
		}

		#[test]
		fn same_string_if_string_length_is_less_than_shrink_length() {
			assert_eq!("aaabbb", shrink("aaabbb", 9));
		}

		#[test]
		fn same_string_if_string_length_is_equal_to_shrink_length() {
			assert_eq!("aaabbbccc", shrink("aaabbbccc", 9));
		}
	}

	mod config {
		mod new {
			use super::super::super::*;

			#[test]
			fn windows_defaults_to_empty() {
				let config = Config::new();
				assert!(config.windows.len() == 0);
			}
		}

		mod load {
			use super::super::super::*;

			#[test]
			fn single_window() {
				let temp_dir = assert_fs::TempDir::new().unwrap();
				let config_file = temp_dir.child("test.json");
				config_file
					.write_str(r#"{"windows": [{"x": 100}]}"#)
					.unwrap();
				let config = Config::load(config_file.path().to_str().unwrap());
				assert!(config.is_ok());
				let config = config.unwrap();
				assert!(config.windows[0].x.is_some());
				assert_eq!(100, config.windows[0].x.unwrap());
			}
		}

		mod save {
			use super::super::super::*;

			#[test]
			fn single_window() {
				let temp_dir = assert_fs::TempDir::new().unwrap();
				let config_file = temp_dir.child("test.json");
				config_file.assert(predicate::path::missing());
				let mut config = Config::new();
				let mut window = Window::new();
				window.x = Some(100);
				config.windows.push(window);
				let save_result = config.save(config_file.path().to_str().unwrap());
				assert!(save_result.is_ok());
				config_file.assert(
					r#"{
  "windows": [
    {
      "x": 100
    }
  ]
}"#,
				);
			}
		}

		mod search {
			use super::super::super::*;

			#[test]
			fn matches_window_given_simple_title_only() {
				let config = ConfigBuilder::default()
					.windows(vec![WindowBuilder::default()
						.title(Some("Example Title".to_string()))
						.build()
						.unwrap()])
					.build()
					.unwrap();
				let mut window_state = sys::WindowState::new();
				window_state.title = Some(String::from("Example Title"));
				let actual = config.search(&window_state);
				assert!(actual.is_some());
				assert_eq!("Example Title", &(actual.unwrap().title.unwrap()));
			}

			#[test]
			fn matches_window_given_regex_title_only() {
				let config = ConfigBuilder::default()
					.windows(vec![WindowBuilder::default()
						.title(Some(".*Title".to_string()))
						.build()
						.unwrap()])
					.build()
					.unwrap();
				let mut window_state = sys::WindowState::new();
				window_state.title = Some(String::from("Example Title"));
				let actual = config.search(&window_state);
				assert!(actual.is_some());
				assert_eq!(".*Title", &(actual.unwrap().title.unwrap()));
			}

			#[test]
			fn matches_window_given_simple_process_only() {
				let config = ConfigBuilder::default()
					.windows(vec![WindowBuilder::default()
						.process(Some("example.exe".to_string()))
						.build()
						.unwrap()])
					.build()
					.unwrap();
				let mut window_state = sys::WindowState::new();
				window_state.process = Some(String::from("example.exe"));
				let actual = config.search(&window_state);
				assert!(actual.is_some());
				assert_eq!("example.exe", actual.unwrap().process.unwrap());
			}

			#[test]
			fn matches_window_given_regex_process_only() {
				let config = ConfigBuilder::default()
					.windows(vec![WindowBuilder::default()
						.process(Some(".*.exe".to_string()))
						.build()
						.unwrap()])
					.build()
					.unwrap();
				let mut window_state = sys::WindowState::new();
				window_state.process = Some(String::from("example.exe"));
				let actual = config.search(&window_state);
				assert!(actual.is_some());
				assert_eq!(".*.exe", actual.unwrap().process.unwrap());
			}

			#[test]
			fn matches_window_given_regex_process_and_regex_title() {
				let config = ConfigBuilder::default()
					.windows(vec![WindowBuilder::default()
						.title(Some(".*Title".to_string()))
						.process(Some(".*.exe".to_string()))
						.build()
						.unwrap()])
					.build()
					.unwrap();
				let mut window_state = sys::WindowState::new();
				window_state.title = Some(String::from("Window Title"));
				window_state.process = Some(String::from("example.exe"));
				let actual = config.search(&window_state);
				assert!(actual.is_some());
				let actual = actual.unwrap();
				assert_eq!(".*Title", actual.title.unwrap());
				assert_eq!(".*.exe", actual.process.unwrap());
			}

			#[test]
			fn preserves_order_given_same_title_with_latter_defining_process() {
				let mut config = Config::new();
				let mut window_1 = Window::new();
				let mut window_2 = Window::new();
				window_1.title = Some(String::from("Example Title"));
				window_2.title = Some(String::from("Example Title"));
				window_2.process = Some(String::from("example.exe"));
				config.windows.push(window_1);
				config.windows.push(window_2);
				let mut window_state = sys::WindowState::new();
				window_state.title = Some(String::from("Example Title"));
				window_state.process = Some(String::from("example.exe"));
				let actual = config.search(&window_state);
				assert!(actual.is_some());
				let actual = actual.unwrap();
				assert!(actual.title.is_some());
				assert!(actual.process.is_none());
				assert_eq!("Example Title", actual.title.unwrap());
			}
		}

		mod window_at {
			use super::super::super::*;

			#[test]
			fn gets_first() {
				let mut config = ConfigBuilder::default()
					.windows(vec![WindowBuilder::default().build().unwrap()])
					.build()
					.unwrap();
				let window = config.window_at(0);
				assert!(window.is_ok());
			}

			#[test]
			fn error_if_out_of_bounds() {
				let mut config = ConfigBuilder::default().windows(vec![]).build().unwrap();
				let window = config.window_at(0);
				assert!(window.is_err());
			}
		}
	}
}

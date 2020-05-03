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

use prettytable::{color, format, Attr, Cell, Row, Table};
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

    pub fn print_windows(&self) {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_COLSEP);
        table.add_row(Row::new(vec![
            Cell::new("Title").style_spec("c"),
            Cell::new("Process").style_spec("c"),
            Cell::new("Position").style_spec("c"),
            Cell::new("Dimension").style_spec("c"),
        ]));
        for w in &self.windows {
            let mut row = Row::new(vec![]);
            if let Some(title) = &w.title {
                row.add_cell(Cell::new(&shrink(&title, 32))
                    .with_style(Attr::ForegroundColor(color::RED)));
            } else {
                row.add_cell(Cell::new(""));
            }
            if let Some(process) = &w.process {
                row.add_cell(Cell::new(&shrink(&process, 64))
                    .with_style(Attr::ForegroundColor(color::GREEN)));
            } else {
                row.add_cell(Cell::new(""));
            }
            match (w.x, w.y) {
                (Some(x), Some(y)) => row.add_cell(Cell::new(
                    &format!("({}, {})", x, y))),
                (None, Some(y)) => row.add_cell(Cell::new(
                    &format!("(null, {})", y))),
                (Some(x), None) => row.add_cell(Cell::new(
                    &format!("({}, null)", x))),
                _ => row.add_cell(Cell::new("")),
            }
            match (w.w, w.h) {
                (Some(w), Some(h)) => row.add_cell(Cell::new(
                    &format!("{} x {}", w, h))),
                (None, Some(h)) => row.add_cell(Cell::new(
                    &format!("null x {}", h))),
                (Some(w), None) => row.add_cell(Cell::new(
                    &format!("{} x null", w))),
                _ => row.add_cell(Cell::new("")),
            }
            table.add_row(row);
        }
        table.printstd();
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
	if the_string.chars().count() > shrink_len {
		let mut shrinked = String::new();
		if shrink_len % 2 == 0 {
			for (i, c) in the_string.chars().enumerate() {
				shrinked.push(c);
				if i >= (shrink_len / 2 - 2) - 1 {
					break;
				}
			}
		} else {
			for (i, c) in the_string.chars().enumerate() {
				shrinked.push(c);
				if i >= (shrink_len / 2 - 1) - 1 {
					break;
				}
			}
		}
		shrinked.push_str("...");
		for (i, c) in the_string.chars().enumerate() {
			if i >= (the_string.len() - (shrink_len / 2) + 1) {
				shrinked.push(c);
			}
		}
		shrinked
	} else {
		the_string.to_string()
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

		#[test]
		fn handles_unicode_char_on_char_boundary() {
			// Fixes the following panic error:
			// panicked at 'byte index 9 is not a char boundary; it is inside '’' (bytes 7..10) of `aa‘bb’cc`'
			let title_with_unicode = "aa‘bb’cc";
			assert_eq!(title_with_unicode, shrink(title_with_unicode, 8));
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

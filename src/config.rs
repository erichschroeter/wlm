use prettytable::{color, format, Attr, Cell, Row, Table};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::path::PathBuf;
use reflection::Reflection;

use crate::{error::Error, get_dimensions_string, get_position_string, shrink, Result};

pub trait Properties {
	fn properties(&self) -> [String];
}

/// Encapsulates the attributes for positioning and resizing windows.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Builder, Reflection)]
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

/// Provides accessor methods for fields used to search for a [Window] in a [Config].
pub trait Searchable {
	fn get_title(&self) -> &str;
	fn get_process(&self) -> &str;
}

impl Searchable for Window {
	fn get_process(&self) -> &str {
		match &self.process {
			Some(process) => &process,
			None => "",
		}
	}

	fn get_title(&self) -> &str {
		match &self.title {
			Some(title) => &title,
			None => "",
		}
	}
}

fn get_window_table(windows: &[Window]) -> Table {
	let mut table = Table::new();
	table.set_format(*format::consts::FORMAT_NO_COLSEP);
	table.add_row(Row::new(vec![
		Cell::new("Title").style_spec("c"),
		Cell::new("Process").style_spec("c"),
		Cell::new("Position").style_spec("c"),
		Cell::new("Dimension").style_spec("c"),
	]));
	for w in windows {
		let mut row = Row::empty();
		if let Some(title) = &w.title {
			row.add_cell(
				Cell::new(&shrink(title, 32)).with_style(Attr::ForegroundColor(color::RED)),
			);
		} else {
			row.add_cell(Cell::default());
		}
		if let Some(process) = &w.process {
			row.add_cell(
				Cell::new(&shrink(process, 64)).with_style(Attr::ForegroundColor(color::GREEN)),
			);
		} else {
			row.add_cell(Cell::default());
		}
		row.add_cell(Cell::new(&get_position_string(w.x, w.y)));
		row.add_cell(Cell::new(&get_dimensions_string(w.w, w.h)));
		table.add_row(row);
	}
	table
}

/// Prints a list of [Window] to any [std::io::Write]able output.
pub fn print_windows<T>(windows: &[Window], out: &mut T)
where
	T: std::io::Write + ?Sized,
{
	let _ = get_window_table(windows).print(out);
}

/// Prints a list of [Window] to [std::io::stdout] (delegated via [prettytable::Table::printstd]).
pub fn print_windows_tty(windows: &[Window]) {
	let _ = get_window_table(windows).printstd();
}

/// Encapsulates a set of [Window] and how to reposition and resize them.
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

	// pub fn search(&self, platform_window: &sys::WindowState) -> Option<Window> {
	pub fn search<T: Searchable>(&self, searchable: &T) -> Option<Window> {
		for cfg_window in &self.windows {
			match (&cfg_window.title, &cfg_window.process) {
				(Some(cfg_title), Some(cfg_process)) => {
					match (Regex::new(cfg_title), Regex::new(cfg_process)) {
						(Ok(title_re), Ok(process_re)) => {
							if title_re.is_match(searchable.get_title())
								&& process_re.is_match(searchable.get_process())
							{
								return Some(cfg_window.clone());
							}
						}
						_ => {}
					}
				}
				(Some(cfg_title), None) => {
					if let Ok(re) = Regex::new(cfg_title) {
						if re.is_match(searchable.get_title()) {
							return Some(cfg_window.clone());
						}
					}
				}
				(None, Some(cfg_process)) => {
					if let Ok(re) = Regex::new(cfg_process) {
						if re.is_match(searchable.get_process()) {
							return Some(cfg_window.clone());
						}
					}
				}
				_ => {}
			}
		}
		None
	}

	/// Returns a tuple of the index and property string.
	/// 
	/// Given the following example:
	/// 
	///     windows.0.title
	/// 
	/// the returned value is (0, "title").
	pub fn parse_property_string(the_string: &str) -> Result<(usize, String)> {
		let tokens: Vec<&str> = the_string.split(".").collect();
		match tokens.len() {
			3 => match tokens[0] {
				"windows" => {
					if let Ok(index) = tokens[1].parse::<usize>() {
						for child in Window::schemata().root() {
							// Use the reflection crate to check for property.
							for member in child.iter() {
								if member.data.id() == tokens[2].to_string() {
									return Ok((index, tokens[2].to_string()));
								}
							}
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

#[cfg(test)]
mod tests {
	use assert_fs::prelude::*;
	use predicates::prelude::*;

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
			use super::super::*;

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
			use super::super::*;

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
			use crate::platform as sys;

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

		mod parse_property_string {
			use super::super::super::*;

			#[test]
			fn windows_0_doesnotexist() {
				assert!(Config::parse_property_string("windows.0.doesnotexist").is_err());
				assert_eq!(
					Error::InvalidProperty,
					Config::parse_property_string("windows.0.doesnotexist").unwrap_err()
				);
			}

			#[test]
			fn windows_0_x() {
				assert_eq!(
					(0, "x".to_string()),
					Config::parse_property_string("windows.0.x").unwrap()
				);
			}

			#[test]
			fn windows_0_y() {
				assert_eq!(
					(0, "y".to_string()),
					Config::parse_property_string("windows.0.y").unwrap()
				);
			}

			#[test]
			fn windows_0_w() {
				assert_eq!(
					(0, "w".to_string()),
					Config::parse_property_string("windows.0.w").unwrap()
				);
			}

			#[test]
			fn windows_0_h() {
				assert_eq!(
					(0, "h".to_string()),
					Config::parse_property_string("windows.0.h").unwrap()
				);
			}

			#[test]
			fn windows_0_title() {
				assert_eq!(
					(0, "title".to_string()),
					Config::parse_property_string("windows.0.title").unwrap()
				);
			}

			#[test]
			fn windows_0_process() {
				assert_eq!(
					(0, "process".to_string()),
					Config::parse_property_string("windows.0.process").unwrap()
				);
			}
		}
	}
}

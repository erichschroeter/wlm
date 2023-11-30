// extern crate clap;
// extern crate directories;

// use clap::{App, Arg, SubCommand};
// use directories::ProjectDirs;
// use exitfailure::ExitFailure;
// use failure::ResultExt;
// use std::env;
// use std::path::Path;
// use wlm::{Layout, WindowBuilder, WindowManager};

// fn main() -> Result<(), ExitFailure> {
// 	let matches = App::new("wlm")
// 		.version("0.3.0")
// 		.about("Move and resize windows.")
// 		.arg(
// 			Arg::with_name("file")
// 				.short("f")
// 				.long("file")
// 				.value_name("config-file")
// 				.help("Use given config file instead of the default")
// 				.takes_value(true)
// 		)
// 		.subcommand(
// 			SubCommand::with_name("ls")
// 				.about(
// 					"Lists active windows and their properies.",
// 				)
// 				.arg(
// 					Arg::with_name("as-json")
// 						.help("Change output format of active windows in JSON")
// 						.long("as-json"),
// 				),
// 		)
// 		.subcommand(
// 			SubCommand::with_name("init")
// 				.about("Create a config")
// 				.arg(
// 					Arg::with_name("force")
// 						.help("Overwrite existing config")
// 						.long("force"),
// 				),
// 		)
// 		.subcommand(
// 			SubCommand::with_name("config")
// 				.about("Modify or view a config. View a config by running without specifying <config-option> argument.")
// 				.arg(
// 					Arg::with_name("reset")
// 						.help("Clears the <config-option>")
// 						.long("reset"),
// 				)
// 				.arg(
// 					Arg::with_name("config-option")
// 						.long("config-option")
// 						.help("Get/Set the value")
// 						.display_order(1)
// 						.index(1)
// 				)
// 				.arg(
// 					Arg::with_name("value")
// 						.long("value")
// 						.help("Corresponding value for <config-option>")
// 						.index(2)
// 				)
// 		)
// 		.subcommand(
// 			SubCommand::with_name("add")
// 				.about("Add a window to the config")
// 				.arg(
// 					Arg::with_name("title")
// 						.short("t")
// 						.long("title")
// 						.help("Set the title value")
// 						.takes_value(true)
// 				)
// 				.arg(
// 					Arg::with_name("process")
// 						.short("p")
// 						.long("process")
// 						.help("Set the process value")
// 						.takes_value(true)
// 				)
// 				.arg(
// 					Arg::with_name("x")
// 						.short("x")
// 						.help("Set the x value")
// 						.takes_value(true)
// 						.allow_hyphen_values(true)
// 				)
// 				.arg(
// 					Arg::with_name("y")
// 						.short("y")
// 						.help("Set the y value")
// 						.takes_value(true)
// 						.allow_hyphen_values(true)
// 				)
// 				.arg(
// 					Arg::with_name("w")
// 						.short("w")
// 						.long("width")
// 						.help("Set the w value")
// 						.takes_value(true)
// 						.allow_hyphen_values(true)
// 				)
// 				.arg(
// 					Arg::with_name("h")
// 						.short("H")
// 						.long("height")
// 						.help("Set the height value")
// 						.takes_value(true)
// 						.allow_hyphen_values(true)
// 				)
// 		)
// 		.subcommand(
// 			SubCommand::with_name("apply")
// 				.about("Apply a profile to active windows"),
// 		)
// 		.get_matches();
// 	let mut config_path = std::path::PathBuf::new();
// 	if let Some(file_arg) = matches.value_of("file") {
// 		config_path = Path::new(file_arg).to_path_buf();
// 	} else if let Some(env_var_file) = env::var_os("WLM_DEFAULT_CONFIG") {
// 		config_path = Path::new(&env_var_file).to_path_buf();
// 	} else if let Some(config_dir) = ProjectDirs::from("com", "wlm", "wlm") {
// 		config_path = config_dir.config_dir().join("default.json");
// 	}
// 	match matches.subcommand() {
// 		("ls", Some(_matches)) => {
// 			WindowManager::new(None).print();
// 		}
// 		("init", Some(matches)) => {
// 			if matches.is_present("force") {
// 				Layout::default()
// 					.save_force(config_path.to_str().unwrap())
// 					.with_context(|_| {
// 						format!("Saving with --force '{}'", config_path.to_str().unwrap())
// 					})?;
// 			} else {
// 				// File & e   : create if and only --force
// 				// Dir  & e   : create default.json
// 				// File & dne : create
// 				// Dir  & dne : create default.json
// 				if config_path.exists() {
// 					if config_path.is_file() {
// 						let error = Err(failure::err_msg(
// 							"Use --force if the intent is to overwrite",
// 						));
// 						return Err(error.context(format!(
// 							"Layout already exists: {}",
// 							config_path.to_str().unwrap()
// 						))?);
// 					} else {
// 						std::fs::create_dir_all(&config_path)?;
// 						let config_path = config_path.join("default.json");
// 						Layout::default()
// 							.save(config_path.to_str().unwrap())
// 							.with_context(|_| {
// 								format!("Saving '{}'", config_path.to_str().unwrap())
// 							})?;
// 					}
// 				} else {
// 					if config_path.extension().is_some() {
// 						if let Some(dir) = config_path.parent() {
// 							std::fs::create_dir_all(dir)?;
// 						}
// 						Layout::default()
// 							.save(config_path.to_str().unwrap())
// 							.with_context(|_| {
// 								format!("Saving '{}'", config_path.to_str().unwrap())
// 							})?;
// 					} else {
// 						std::fs::create_dir_all(&config_path)?;
// 						let config_path = config_path.join("default.json");
// 						Layout::default()
// 							.save(config_path.to_str().unwrap())
// 							.with_context(|_| {
// 								format!("Saving '{}'", config_path.to_str().unwrap())
// 							})?;
// 					}
// 				}
// 			}
// 		}
// 		("config", Some(matches)) => {
// 			let mut config = Layout::load(config_path.to_str().unwrap())
// 				.with_context(|_| format!("Loading '{}'", config_path.to_str().unwrap()))?;
// 			match (matches.value_of("config-option"), matches.value_of("value")) {
// 				(Some(config_option), Some(value)) => {
// 					let prop = Layout::parse_property_string(config_option)
// 						.with_context(|_| format!("Unsupported property '{}'", config_option))?;
// 					let window_count = config.windows.len();
// 					let mut window = config.window_at(prop.0).context(match window_count {
// 						1 => format!("Valid index is 0"),
// 						0 => format!("No windows exist"),
// 						_ => format!("Valid index range is 0 to {}", window_count - 1),
// 					})?;
// 					match prop.1.as_str() {
// 						"x" => {
// 							let x = value.parse::<i32>()?;
// 							window.x = Some(x);
// 							config.save(config_path.to_str().unwrap())?;
// 						}
// 						"y" => {
// 							let y = value.parse::<i32>()?;
// 							window.y = Some(y);
// 							config.save(config_path.to_str().unwrap())?;
// 						}
// 						"w" => {
// 							let w = value.parse::<i32>()?;
// 							window.w = Some(w);
// 							config.save(config_path.to_str().unwrap())?;
// 						}
// 						"h" => {
// 							let h = value.parse::<i32>()?;
// 							window.h = Some(h);
// 							config.save(config_path.to_str().unwrap())?;
// 						}
// 						_ => {}
// 					}
// 				}
// 				(Some(config_option), None) => {
// 					let prop = Layout::parse_property_string(config_option)
// 						.with_context(|_| format!("Unsupported property '{}'", config_option))?;
// 					let window_count = config.windows.len();
// 					let mut window = config.window_at(prop.0).context(match window_count {
// 						1 => format!("Valid index is 0"),
// 						0 => format!("No windows exist"),
// 						_ => format!("Valid index range is 0 to {}", window_count - 1),
// 					})?;
// 					match prop.1.as_str() {
// 						"x" => {
// 							if matches.is_present("reset") {
// 								window.x = None;
// 								config.save(config_path.to_str().unwrap())?;
// 							} else if let Some(x) = window.x {
// 								println!("{}", x);
// 							}
// 						}
// 						"y" => {
// 							if matches.is_present("reset") {
// 								window.y = None;
// 								config.save(config_path.to_str().unwrap())?;
// 							} else if let Some(y) = window.y {
// 								println!("{}", y);
// 							}
// 						}
// 						"w" => {
// 							if matches.is_present("reset") {
// 								window.w = None;
// 								config.save(config_path.to_str().unwrap())?;
// 							} else if let Some(w) = window.w {
// 								println!("{}", w);
// 							}
// 						}
// 						"h" => {
// 							if matches.is_present("reset") {
// 								window.h = None;
// 								config.save(config_path.to_str().unwrap())?;
// 							} else if let Some(h) = window.h {
// 								println!("{}", h);
// 							}
// 						}
// 						_ => {}
// 					}
// 				}
// 				_ => {
// 					config.print_windows();
// 				}
// 			}
// 		}
// 		("add", Some(matches)) => {
// 			let mut config = Layout::load(config_path.to_str().unwrap())
// 				.with_context(|_| format!("Loading '{}'", config_path.to_str().unwrap()))?;
// 			let mut window = WindowBuilder::default();
// 			if let Some(value) = matches.value_of("title") {
// 				window.title(Some(value.to_string()));
// 			}
// 			if let Some(value) = matches.value_of("process") {
// 				window.process(Some(value.to_string()));
// 			}
// 			if let Some(value) = matches.value_of("x") {
// 				window.x(Some(value.parse::<i32>()?));
// 			}
// 			if let Some(value) = matches.value_of("y") {
// 				window.y(Some(value.parse::<i32>()?));
// 			}
// 			if let Some(value) = matches.value_of("w") {
// 				window.w(Some(value.parse::<i32>()?));
// 			}
// 			if let Some(value) = matches.value_of("h") {
// 				window.h(Some(value.parse::<i32>()?));
// 			}
// 			config.windows.push(window.build().unwrap());
// 			config
// 				.save(config_path.to_str().unwrap())
// 				.with_context(|_| format!("Saving '{}'", config_path.to_str().unwrap()))?;
// 		}
// 		("apply", Some(_matches)) => {
// 			let config = Layout::load(config_path.to_str().unwrap())
// 				.with_context(|_| format!("Loading '{}'", config_path.to_str().unwrap()))?;
// 			WindowManager::new(Some(&config)).layout();
// 		}
// 		_ => {}
// 	}
// 	Ok(())
// }

use std::path::Path;

use clap::{value_parser, Arg, ArgMatches};
use directories::ProjectDirs;
use log::{debug, LevelFilter};

use cor_args::{ArgHandler, DefaultHandler, EnvHandler, FileHandler, Handler};
use prettytable::{color, format, Attr, Cell, Row, Table};
use wlm::{
	default_window_provider,
	layout::{Format, Layout, LayoutBuilder},
	shrink_left, shrink_right, WindowProvider,
};

/// Sets up logging based on the specified verbosity level.
///
/// This function initializes the logging framework using `env_logger` crate.
/// The verbosity level determines the amount of log output that will be displayed.
///
/// # Examples
///
/// ```
/// use crate::setup_logging;
///
/// setup_logging("debug");
/// ```
///
/// # Arguments
///
/// * `verbosity` - A string slice representing the desired verbosity level.
///   Valid values are "off", "error", "warn", "info", "debug", and "trace".
///   If an invalid value is provided, the default level will be set to "info".
///
/// # Dependencies
///
/// This function depends on the following crates:
///
/// - `env_logger` - For setting up logging.
/// - `log` - For defining log levels.
///
/// # Panics
///
/// This function will panic if the `verbosity` string cannot be parsed into a `LevelFilter`.
///
/// # Notes
///
/// It is recommended to call this function early in the program to set up logging
/// before any log messages are generated.
///
fn setup_logging(verbosity: &str) {
	env_logger::builder()
		.filter(None, verbosity.parse().unwrap_or(LevelFilter::Info))
		.init();
}

fn ls_table() {
	let mut table = Table::new();
	table.set_format(*format::consts::FORMAT_CLEAN);
	table.add_row(Row::new(vec![
		Cell::new("Title").style_spec("c"),
		Cell::new("Process").style_spec("c"),
		Cell::new("Point").style_spec("l"),
		Cell::new("Dimension").style_spec("l"),
	]));
	for s in default_window_provider().screens() {
		for w in s.windows {
			table.add_row(Row::new(vec![
				Cell::new(&shrink_right(
					w.title.as_ref().unwrap_or(&"".to_string()),
					32,
				))
				.with_style(Attr::ForegroundColor(color::RED)),
				Cell::new(&shrink_left(
					w.process.as_ref().unwrap_or(&"".to_string()),
					64,
				))
				.with_style(Attr::ForegroundColor(color::GREEN)),
				Cell::new(&format!(
					"({}, {})",
					w.x.as_ref().unwrap_or(&"0".to_string()),
					w.y.as_ref().unwrap_or(&"0".to_string())
				)),
				Cell::new(&format!(
					"{} x {}",
					w.w.as_ref().unwrap_or(&"0".to_string()),
					w.h.as_ref().unwrap_or(&"0".to_string())
				)),
			]));
		}
	}
	table.printstd();
}

fn ls_yaml(out: &'_ mut dyn std::io::Write) {
	let screens = default_window_provider().screens();
	let layout = LayoutBuilder::default().screens(screens).build().unwrap();
	write!(out, "{}", serde_yaml::to_string(&layout).unwrap()).expect("Failed writing YAML output");
}

fn ls(matches: &ArgMatches) {
	let format = matches
		.get_one::<Format>("format")
		.unwrap_or(&Format::Table);
	log::warn!("Format: {}", format);
	match format {
		Format::Table => ls_table(),
		Format::Yaml => ls_yaml(&mut std::io::stdout()),
	}
}

fn load_layout<S: AsRef<str>>(name: S, default_layout: Layout) -> Layout {
	// Determine where to search for layouts via LAYOUT_PATH
	let layout_path = if let Some(config_dir) = ProjectDirs::from("com", "wlm", "wlm") {
		config_dir.config_dir().join("layouts")
	} else {
		Path::new("~/.config/wlm/layouts").into()
	};
	log::debug!("WLM_LAYOUT_PATH: {}", layout_path.display());

	let layout_file = Path::new(&layout_path)
		.join(name.as_ref())
		.with_extension("yml");
	if layout_file.exists() {
		let file = config::File::from(layout_file.as_path());
		let layout = config::Config::builder()
			.add_source(file.required(true))
			.build()
			.unwrap();
		let layout = layout
			.try_deserialize::<Layout>()
			.expect(&format!("Failed to load layout: {}", layout_file.display()));
		layout
	} else {
		default_layout
	}
}

fn layout(matches: &ArgMatches) {
	println!("Running layout: {:?}", matches);
	// Determine the layout to load from LAYOUT_PATH
	let layout_name = ArgHandler::new(&matches)
		.next(
			EnvHandler::new()
				.prefix("WLM_")
				.next(DefaultHandler::new("default").into())
				.into(),
		)
		.handle_request("layout_name")
		.unwrap();
	log::debug!("layout_name = {}", layout_name);

	let layout = load_layout(layout_name, Layout::default());
	debug!("Applying layout: {:?}", layout);
	default_window_provider().layout(&layout);
}

struct App {
	args: clap::Command,
}

impl App {
	pub fn new() -> Self {
		App {
			args: clap::Command::new("wlm")
				.version("v0.4.0")
				.author("Erich Schroeter <erich.schroeter@gmail.com>")
				.about("A command-line tool to move and resize windows")
				.arg(
					Arg::new("verbosity")
						.short('v')
						.long("verbosity")
						.value_name("VERBOSITY")
						// .default_value(Settings::default().verbose)
						.help("Set the logging verbosity level.")
						.long_help("Choices: [off, error, warn, info, debug, trace]"),
				)
				.infer_subcommands(true)
				.arg_required_else_help(true)
				.subcommand(
					clap::Command::new("ls")
						.about("List windows and associated attributes")
						.arg(
							Arg::new("format")
								.help("Output as specified format")
								.long_help(format!(
									"Output as specified format {:?}",
									wlm::layout::FORMAT_NAMES
								))
								.short('f')
								.long("format")
								.value_name("FORMAT")
								.default_value("table")
								.value_parser(value_parser!(Format))
								.required(false),
						),
				)
				.subcommand(
					clap::Command::new("layout")
						.about("Moves windows around determined by specified layout")
						.arg(
							Arg::new("layout")
								.help("Path the layout file")
								.required(false),
						),
				),
		}
	}

	pub fn run_with_args<I, T>(&mut self, args: I) -> Result<(), Box<dyn std::error::Error>>
	where
		I: IntoIterator<Item = T>,
		T: Into<std::ffi::OsString> + Clone,
	{
		let matches = self.args.clone().get_matches_from(args);
		let verbosity_handler = ArgHandler::new(&matches).next(
			EnvHandler::new()
				.prefix("WLM_")
				.next(
					FileHandler::new("~/.config/wlm/verbosity")
						.next(DefaultHandler::new("info").into())
						.into(),
				)
				.into(),
		);
		if let Some(verbosity) = verbosity_handler.handle_request("verbosity") {
			setup_logging(&verbosity);
		}

		match matches.subcommand() {
			Some(("ls", sub_m)) => ls(sub_m),
			Some(("layout", sub_m)) => layout(sub_m),
			_ => eprintln!("Invalid subcommand!"),
		}
		Ok(())
	}

	pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		self.run_with_args(std::env::args().into_iter())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[ignore]
	#[test]
	fn test_run_with_args() {
		assert_eq!(
			Some(()),
			App::new().run_with_args(&vec!["fixme.exe", "ls"]).ok()
		);
	}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	App::new().run()
}

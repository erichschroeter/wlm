extern crate clap;
extern crate directories;

use clap::{App, Arg, SubCommand};
use directories::ProjectDirs;
use exitfailure::ExitFailure;
use failure::ResultExt;
use std::env;
use std::path::Path;
use window_layout_manager::{config::Config, config::WindowBuilder, wm::WindowManager};

fn main() -> Result<(), ExitFailure> {
	let matches = App::new("wlm")
		.version("0.3.0")
		.about("Move and resize windows.")
		.arg(
			Arg::with_name("file")
				.short("f")
				.long("file")
				.value_name("config-file")
				.help("Use given config file instead of the default")
				.takes_value(true)
		)
		.subcommand(
			SubCommand::with_name("ls")
				.about(
					"Lists active windows and their properies.",
				)
				.arg(
					Arg::with_name("as-json")
						.help("Change output format of active windows in JSON")
						.long("as-json"),
				),
		)
		.subcommand(
			SubCommand::with_name("init")
				.about("Create a config")
				.arg(
					Arg::with_name("force")
						.help("Overwrite existing config")
						.long("force"),
				),
		)
		.subcommand(
			SubCommand::with_name("config")
				.about("Modify or view a config. View a config by running without specifying <config-option> argument.")
				.arg(
					Arg::with_name("reset")
						.help("Clears the <config-option>")
						.long("reset"),
				)
				.arg(
					Arg::with_name("config-option")
						.long("config-option")
						.help("Get/Set the value")
						.display_order(1)
						.index(1)
				)
				.arg(
					Arg::with_name("value")
						.long("value")
						.help("Corresponding value for <config-option>")
						.index(2)
				)
		)
		.subcommand(
			SubCommand::with_name("add")
				.about("Add a window to the config")
				.arg(
					Arg::with_name("title")
						.short("t")
						.long("title")
						.help("Set the title value")
						.takes_value(true)
				)
				.arg(
					Arg::with_name("process")
						.short("p")
						.long("process")
						.help("Set the process value")
						.takes_value(true)
				)
				.arg(
					Arg::with_name("x")
						.short("x")
						.help("Set the x value")
						.takes_value(true)
						.allow_hyphen_values(true)
				)
				.arg(
					Arg::with_name("y")
						.short("y")
						.help("Set the y value")
						.takes_value(true)
						.allow_hyphen_values(true)
				)
				.arg(
					Arg::with_name("w")
						.short("w")
						.long("width")
						.help("Set the w value")
						.takes_value(true)
						.allow_hyphen_values(true)
				)
				.arg(
					Arg::with_name("h")
						.short("H")
						.long("height")
						.help("Set the height value")
						.takes_value(true)
						.allow_hyphen_values(true)
				)
		)
		.subcommand(
			SubCommand::with_name("apply")
				.about("Apply a profile to active windows"),
		)
		.get_matches();
	let mut config_path = std::path::PathBuf::new();
	if let Some(file_arg) = matches.value_of("file") {
		config_path = Path::new(file_arg).to_path_buf();
	} else if let Some(env_var_file) = env::var_os("WLM_DEFAULT_CONFIG") {
		config_path = Path::new(&env_var_file).to_path_buf();
	} else if let Some(config_dir) = ProjectDirs::from("com", "wlm", "wlm") {
		config_path = config_dir.config_dir().join("default.json");
	}
	match matches.subcommand() {
		("ls", Some(_matches)) => {
			WindowManager::new(None).print();
			for m in WindowManager::new(None).monitors() {
				println!("{:?}", m);
			}
		}
		("init", Some(matches)) => {
			if matches.is_present("force") {
				Config::default()
					.save_force(config_path.to_str().unwrap())
					.with_context(|_| {
						format!("Saving with --force '{}'", config_path.to_str().unwrap())
					})?;
			} else {
				// File & e   : create if and only --force
				// Dir  & e   : create default.json
				// File & dne : create
				// Dir  & dne : create default.json
				if config_path.exists() {
					if config_path.is_file() {
						let error = Err(failure::err_msg(
							"Use --force if the intent is to overwrite",
						));
						return Err(error.context(format!(
							"Config already exists: {}",
							config_path.to_str().unwrap()
						))?);
					} else {
						std::fs::create_dir_all(&config_path)?;
						let config_path = config_path.join("default.json");
						Config::default()
							.save(config_path.to_str().unwrap())
							.with_context(|_| {
								format!("Saving '{}'", config_path.to_str().unwrap())
							})?;
					}
				} else {
					if config_path.extension().is_some() {
						if let Some(dir) = config_path.parent() {
							std::fs::create_dir_all(dir)?;
						}
						Config::default()
							.save(config_path.to_str().unwrap())
							.with_context(|_| {
								format!("Saving '{}'", config_path.to_str().unwrap())
							})?;
					} else {
						std::fs::create_dir_all(&config_path)?;
						let config_path = config_path.join("default.json");
						Config::default()
							.save(config_path.to_str().unwrap())
							.with_context(|_| {
								format!("Saving '{}'", config_path.to_str().unwrap())
							})?;
					}
				}
			}
		}
		("config", Some(matches)) => {
			let mut config = Config::load(config_path.to_str().unwrap())
				.with_context(|_| format!("Loading '{}'", config_path.to_str().unwrap()))?;
			match (matches.value_of("config-option"), matches.value_of("value")) {
				(Some(config_option), Some(value)) => {
					let prop = Config::parse_property_string(config_option)
						.with_context(|_| format!("Unsupported property '{}'", config_option))?;
					let window_count = config.windows.len();
					let mut window = config.window_at(prop.0).context(match window_count {
						1 => format!("Valid index is 0"),
						0 => format!("No windows exist"),
						_ => format!("Valid index range is 0 to {}", window_count - 1),
					})?;
					match prop.1.as_str() {
						"x" => {
							let x = value.parse::<i32>()?;
							window.x = Some(x);
							config.save(config_path.to_str().unwrap())?;
						}
						"y" => {
							let y = value.parse::<i32>()?;
							window.y = Some(y);
							config.save(config_path.to_str().unwrap())?;
						}
						"w" => {
							let w = value.parse::<i32>()?;
							window.w = Some(w);
							config.save(config_path.to_str().unwrap())?;
						}
						"h" => {
							let h = value.parse::<i32>()?;
							window.h = Some(h);
							config.save(config_path.to_str().unwrap())?;
						}
						_ => {}
					}
				}
				(Some(config_option), None) => {
					let prop = Config::parse_property_string(config_option)
						.with_context(|_| format!("Unsupported property '{}'", config_option))?;
					let window_count = config.windows.len();
					let mut window = config.window_at(prop.0).context(match window_count {
						1 => format!("Valid index is 0"),
						0 => format!("No windows exist"),
						_ => format!("Valid index range is 0 to {}", window_count - 1),
					})?;
					match prop.1.as_str() {
						"x" => {
							if matches.is_present("reset") {
								window.x = None;
								config.save(config_path.to_str().unwrap())?;
							} else if let Some(x) = window.x {
								println!("{}", x);
							}
						}
						"y" => {
							if matches.is_present("reset") {
								window.y = None;
								config.save(config_path.to_str().unwrap())?;
							} else if let Some(y) = window.y {
								println!("{}", y);
							}
						}
						"w" => {
							if matches.is_present("reset") {
								window.w = None;
								config.save(config_path.to_str().unwrap())?;
							} else if let Some(w) = window.w {
								println!("{}", w);
							}
						}
						"h" => {
							if matches.is_present("reset") {
								window.h = None;
								config.save(config_path.to_str().unwrap())?;
							} else if let Some(h) = window.h {
								println!("{}", h);
							}
						}
						_ => {}
					}
				}
				_ => {
					config.print_windows();
				}
			}
		}
		("add", Some(matches)) => {
			let mut config = Config::load(config_path.to_str().unwrap())
				.with_context(|_| format!("Loading '{}'", config_path.to_str().unwrap()))?;
			let mut window = WindowBuilder::default();
			if let Some(value) = matches.value_of("title") {
				window.title(Some(value.to_string()));
			}
			if let Some(value) = matches.value_of("process") {
				window.process(Some(value.to_string()));
			}
			if let Some(value) = matches.value_of("x") {
				window.x(Some(value.parse::<i32>()?));
			}
			if let Some(value) = matches.value_of("y") {
				window.y(Some(value.parse::<i32>()?));
			}
			if let Some(value) = matches.value_of("w") {
				window.w(Some(value.parse::<i32>()?));
			}
			if let Some(value) = matches.value_of("h") {
				window.h(Some(value.parse::<i32>()?));
			}
			config.windows.push(window.build().unwrap());
			config
				.save(config_path.to_str().unwrap())
				.with_context(|_| format!("Saving '{}'", config_path.to_str().unwrap()))?;
		}
		("apply", Some(_matches)) => {
			let config = Config::load(config_path.to_str().unwrap())
				.with_context(|_| format!("Loading '{}'", config_path.to_str().unwrap()))?;
			WindowManager::new(Some(&config)).layout();
		}
		_ => {}
	}
	Ok(())
}

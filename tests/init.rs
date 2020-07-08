extern crate directories;

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::env;

#[test]
fn init_command_creates_given_file_does_not_exist() {
	let temp_dir = assert_fs::TempDir::new().unwrap();
	let config_file = temp_dir.child("test.json");
	config_file.assert(predicate::path::missing());
	let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
	let _output = cmd
		.args(&["-f", config_file.path().to_str().unwrap(), "init"])
		.output()
		.expect("failed to get command output");
	config_file.assert(predicate::path::exists());
	config_file.assert(
		r#"{
  "windows": []
}"#,
	);
	temp_dir.close().unwrap();
}

#[test]
fn init_command_creates_given_dir_does_not_exist() {
	let temp_dir = assert_fs::TempDir::new().unwrap();
	let config_file = temp_dir.child("dir_does_not_exist/");
	config_file.assert(predicate::path::missing());
	let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
	let _output = cmd
		.args(&["-f", config_file.path().to_str().unwrap(), "init"])
		.output()
		.expect("failed to get command output");
	config_file.assert(predicate::path::exists());
	let config_file = config_file.child("default.json");
	config_file.assert(
		r#"{
  "windows": []
}"#,
	);
	temp_dir.close().unwrap();
}

#[test]
fn init_command_overwrites_given_file_exists_with_force_option() {
	let temp_dir = assert_fs::TempDir::new().unwrap();
	let config_file = temp_dir.child("test.json");
	config_file
		.write_str(r#"{"windows": [{"x": 100}]}"#)
		.unwrap();
	let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
	let _output = cmd
		.args(&[
			"-f",
			config_file.path().to_str().unwrap(),
			"init",
			"--force",
		])
		.output()
		.expect("failed to get command output");
	config_file.assert(
		r#"{
  "windows": []
}"#,
	);
	temp_dir.close().unwrap();
}

#[test]
fn init_command_error_given_file_exists_without_force_option() {
	let temp_dir = assert_fs::TempDir::new().unwrap();
	let config_file = temp_dir.child("test.json");
	config_file
		.write_str(r#"{"windows": [{"x": 100}]}"#)
		.unwrap();
	let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
	let _output = cmd
		.args(&["-f", config_file.path().to_str().unwrap(), "init"])
		.output()
		.expect("failed to get command output");
	cmd.assert().failure();
	config_file.assert(r#"{"windows": [{"x": 100}]}"#);
	temp_dir.close().unwrap();
}

extern crate directories;

use assert_cmd::Command;
use assert_fs::prelude::*;
use std::env;

#[test]
fn add_command_creates_empty_window_given_no_args() {
	let temp_dir = assert_fs::TempDir::new().unwrap();
	let config_file = temp_dir.child("test.json");
	config_file.write_str(r#"{"windows": []}"#).unwrap();
	let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
	let _output = cmd
		.args(&["-f", config_file.path().to_str().unwrap(), "add"])
		.output()
		.expect("failed to get command output");
	config_file.assert(
		r#"{
  "windows": [
    {}
  ]
}"#,
	);
	temp_dir.close().unwrap();
}

#[test]
fn add_command_sets_title() {
	let temp_dir = assert_fs::TempDir::new().unwrap();
	let config_file = temp_dir.child("test.json");
	config_file.write_str(r#"{"windows": []}"#).unwrap();
	let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
	let _output = cmd
		.args(&[
			"-f",
			config_file.path().to_str().unwrap(),
			"add",
			"--title",
			"Window 1",
		])
		.output()
		.expect("failed to get command output");
	config_file.assert(
		r#"{
  "windows": [
    {
      "title": "Window 1"
    }
  ]
}"#,
	);
	temp_dir.close().unwrap();
}

#[test]
fn add_command_sets_process() {
	let temp_dir = assert_fs::TempDir::new().unwrap();
	let config_file = temp_dir.child("test.json");
	config_file.write_str(r#"{"windows": []}"#).unwrap();
	let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
	let _output = cmd
		.args(&[
			"-f",
			config_file.path().to_str().unwrap(),
			"add",
			"--process",
			"example.exe",
		])
		.output()
		.expect("failed to get command output");
	config_file.assert(
		r#"{
  "windows": [
    {
      "process": "example.exe"
    }
  ]
}"#,
	);
	temp_dir.close().unwrap();
}

#[test]
fn add_command_sets_x() {
	let temp_dir = assert_fs::TempDir::new().unwrap();
	let config_file = temp_dir.child("test.json");
	config_file.write_str(r#"{"windows": []}"#).unwrap();
	let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
	let _output = cmd
		.args(&[
			"-f",
			config_file.path().to_str().unwrap(),
			"add",
			"-x",
			"100",
		])
		.output()
		.expect("failed to get command output");
	config_file.assert(
		r#"{
  "windows": [
    {
      "x": 100
    }
  ]
}"#,
	);
	temp_dir.close().unwrap();
}

#[test]
fn add_command_sets_y() {
	let temp_dir = assert_fs::TempDir::new().unwrap();
	let config_file = temp_dir.child("test.json");
	config_file.write_str(r#"{"windows": []}"#).unwrap();
	let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
	let _output = cmd
		.args(&[
			"-f",
			config_file.path().to_str().unwrap(),
			"add",
			"-y",
			"100",
		])
		.output()
		.expect("failed to get command output");
	config_file.assert(
		r#"{
  "windows": [
    {
      "y": 100
    }
  ]
}"#,
	);
	temp_dir.close().unwrap();
}

#[test]
fn add_command_sets_w() {
	let temp_dir = assert_fs::TempDir::new().unwrap();
	let config_file = temp_dir.child("test.json");
	config_file.write_str(r#"{"windows": []}"#).unwrap();
	let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
	let _output = cmd
		.args(&[
			"-f",
			config_file.path().to_str().unwrap(),
			"add",
			"--width",
			"100",
		])
		.output()
		.expect("failed to get command output");
	config_file.assert(
		r#"{
  "windows": [
    {
      "w": 100
    }
  ]
}"#,
	);
	temp_dir.close().unwrap();
}

#[test]
fn add_command_sets_h() {
	let temp_dir = assert_fs::TempDir::new().unwrap();
	let config_file = temp_dir.child("test.json");
	config_file.write_str(r#"{"windows": []}"#).unwrap();
	let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
	let _output = cmd
		.args(&[
			"-f",
			config_file.path().to_str().unwrap(),
			"add",
			"--height",
			"100",
		])
		.output()
		.expect("failed to get command output");
	config_file.assert(
		r#"{
  "windows": [
    {
      "h": 100
    }
  ]
}"#,
	);
	temp_dir.close().unwrap();
}

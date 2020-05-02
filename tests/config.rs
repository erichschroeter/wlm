extern crate directories;

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::env;

#[test]
fn config_command_error_when_config_is_directory() {
	let temp_dir = assert_fs::TempDir::new().unwrap();
	let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
	let output = cmd
		.args(&["-f", temp_dir.path().to_str().unwrap(), "config"])
		.output()
		.expect("failed to get command output");
	cmd.assert().failure();
	let stderr = String::from_utf8(output.stderr).unwrap();
	assert!(stderr.contains("caused by IO error"));
	temp_dir.close().unwrap();
}

#[test]
fn config_command_error_given_non_existing_file() {
	let temp_dir = assert_fs::TempDir::new().unwrap();
	let config_file = temp_dir.child("does_not_exist.json");
	config_file.assert(predicate::path::missing());
	let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
	let output = cmd
		.args(&["-f", config_file.path().to_str().unwrap(), "config"])
		.output()
		.expect("failed to get command output");
	cmd.assert().failure();
	let stderr = String::from_utf8(output.stderr).unwrap();
	assert!(stderr.contains("caused by IO error"));
	temp_dir.close().unwrap();
}

#[test]
fn config_command_error_given_empty_file() {
	let temp_dir = assert_fs::TempDir::new().unwrap();
	let config_file = temp_dir.child("empty.json");
	config_file.touch().unwrap();
	let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
	let output = cmd
		.args(&["-f", config_file.path().to_str().unwrap(), "config"])
		.output()
		.expect("failed to get command output");
	cmd.assert().failure();
	let stderr = String::from_utf8(output.stderr).unwrap();
	assert!(stderr.contains("caused by Validation error"));
	temp_dir.close().unwrap();
}

#[test]
fn config_command_error_given_invalid_json() {
	let temp_dir = assert_fs::TempDir::new().unwrap();
	let config_file = temp_dir.child("invalid.json");
	config_file.write_str("{[}").unwrap();
	let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
	let output = cmd
		.args(&["-f", config_file.path().to_str().unwrap(), "config"])
		.output()
		.expect("failed to get command output");
	cmd.assert().failure();
	let stderr = String::from_utf8(output.stderr).unwrap();
	assert!(stderr.contains("caused by Validation error"));
	temp_dir.close().unwrap();
}

#[test]
fn reset_removes_attribute() {
	let temp_dir = assert_fs::TempDir::new().unwrap();
	let config_file = temp_dir.child("test.json");
	config_file
		.write_str(r#"{"windows": [{"x": 100, "y": 101}]}"#)
		.unwrap();
	let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
	let _output = cmd
		.args(&[
			"-f",
			config_file.path().to_str().unwrap(),
			"config",
			"windows.0.y",
			"--reset",
		])
		.output()
		.expect("failed to get command output");
	cmd.assert().success();
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

#[cfg(test)]
mod set {
	use super::*;

	#[test]
	fn error_given_index_out_of_range() {
		let temp_dir = assert_fs::TempDir::new().unwrap();
		let config_file = temp_dir.child("test.json");
		config_file.write_str(r#"{"windows": []}"#).unwrap();
		let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
		let output = cmd
			.args(&[
				"-f",
				config_file.path().to_str().unwrap(),
				"config",
				"windows.0.x",
				"100",
			])
			.output()
			.expect("failed to get command output");
		cmd.assert().failure();
		let stderr = String::from_utf8(output.stderr).unwrap();
		assert!(stderr.contains("caused by Invalid index"));
		temp_dir.close().unwrap();
	}

	#[test]
	fn error_given_negative_index() {
		let temp_dir = assert_fs::TempDir::new().unwrap();
		let config_file = temp_dir.child("test.json");
		config_file.write_str(r#"{"windows": []}"#).unwrap();
		let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
		let output = cmd
			.args(&[
				"-f",
				config_file.path().to_str().unwrap(),
				"config",
				"windows.-1.x",
				"100",
			])
			.output()
			.expect("failed to get command output");
		cmd.assert().failure();
		let stderr = String::from_utf8(output.stderr).unwrap();
		assert!(stderr.contains("caused by Invalid index"));
		temp_dir.close().unwrap();
	}

	#[test]
	fn does_not_change_other_properties() {
		let temp_dir = assert_fs::TempDir::new().unwrap();
		let config_file = temp_dir.child("test.json");
		config_file
			.write_str(r#"{"windows": [{"y": 101}]}"#)
			.unwrap();
		let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
		let _output = cmd
			.args(&[
				"-f",
				config_file.path().to_str().unwrap(),
				"config",
				"windows.0.x",
				"100",
			])
			.output()
			.expect("failed to get command output");
		cmd.assert().success();
		config_file.assert(
			r#"{
  "windows": [
    {
      "x": 100,
      "y": 101
    }
  ]
}"#,
		);
		temp_dir.close().unwrap();
	}

	#[test]
	fn error_given_unsupported_property() {
		let temp_dir = assert_fs::TempDir::new().unwrap();
		let config_file = temp_dir.child("test.json");
		config_file.write_str(r#"{"windows": [{}]}"#).unwrap();
		let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
		let output = cmd
			.args(&[
				"-f",
				config_file.path().to_str().unwrap(),
				"config",
				"windows.0.notsupported",
				"100",
			])
			.output()
			.expect("failed to get command output");
		cmd.assert().failure();
		let stderr = String::from_utf8(output.stderr).unwrap();
		assert!(stderr.contains("caused by Invalid property"));
		temp_dir.close().unwrap();
	}

	#[test]
	fn x() {
		let temp_dir = assert_fs::TempDir::new().unwrap();
		let config_file = temp_dir.child("test.json");
		config_file.write_str(r#"{"windows": [{}]}"#).unwrap();
		let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
		let _output = cmd
			.args(&[
				"-f",
				config_file.path().to_str().unwrap(),
				"config",
				"windows.0.x",
				"100",
			])
			.output()
			.expect("failed to get command output");
		cmd.assert().success();
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
	fn y() {
		let temp_dir = assert_fs::TempDir::new().unwrap();
		let config_file = temp_dir.child("test.json");
		config_file.write_str(r#"{"windows": [{}]}"#).unwrap();
		let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
		let _output = cmd
			.args(&[
				"-f",
				config_file.path().to_str().unwrap(),
				"config",
				"windows.0.y",
				"100",
			])
			.output()
			.expect("failed to get command output");
		cmd.assert().success();
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
	fn w() {
		let temp_dir = assert_fs::TempDir::new().unwrap();
		let config_file = temp_dir.child("test.json");
		config_file.write_str(r#"{"windows": [{}]}"#).unwrap();
		let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
		let _output = cmd
			.args(&[
				"-f",
				config_file.path().to_str().unwrap(),
				"config",
				"windows.0.w",
				"100",
			])
			.output()
			.expect("failed to get command output");
		cmd.assert().success();
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
	fn h() {
		let temp_dir = assert_fs::TempDir::new().unwrap();
		let config_file = temp_dir.child("test.json");
		config_file.write_str(r#"{"windows": [{}]}"#).unwrap();
		let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
		let _output = cmd
			.args(&[
				"-f",
				config_file.path().to_str().unwrap(),
				"config",
				"windows.0.h",
				"100",
			])
			.output()
			.expect("failed to get command output");
		cmd.assert().success();
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
}

#[cfg(test)]
mod get {
	use super::*;

	#[test]
	fn error_given_index_out_of_range() {
		let temp_dir = assert_fs::TempDir::new().unwrap();
		let config_file = temp_dir.child("test.json");
		config_file.write_str(r#"{"windows": []}"#).unwrap();
		let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
		let output = cmd
			.args(&[
				"-f",
				config_file.path().to_str().unwrap(),
				"config",
				"windows.0.x",
			])
			.output()
			.expect("failed to get command output");
		let stderr = String::from_utf8(output.stderr).unwrap();
		assert!(stderr.contains("caused by Invalid index"));
		temp_dir.close().unwrap();
	}

	#[test]
	fn x() {
		let temp_dir = assert_fs::TempDir::new().unwrap();
		let config_file = temp_dir.child("test.json");
		config_file
			.write_str(r#"{"windows": [{"x": 100}]}"#)
			.unwrap();
		let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
		let output = cmd
			.args(&[
				"-f",
				config_file.path().to_str().unwrap(),
				"config",
				"windows.0.x",
			])
			.output()
			.expect("failed to get command output");
		cmd.assert().success();
		let stdout = String::from_utf8(output.stdout).unwrap();
		assert_eq!("100\n", stdout);
		temp_dir.close().unwrap();
	}

	#[test]
	fn y() {
		let temp_dir = assert_fs::TempDir::new().unwrap();
		let config_file = temp_dir.child("test.json");
		config_file
			.write_str(r#"{"windows": [{"y": 100}]}"#)
			.unwrap();
		let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
		let output = cmd
			.args(&[
				"-f",
				config_file.path().to_str().unwrap(),
				"config",
				"windows.0.y",
			])
			.output()
			.expect("failed to get command output");
		cmd.assert().success();
		let stdout = String::from_utf8(output.stdout).unwrap();
		assert_eq!("100\n", stdout);
		temp_dir.close().unwrap();
	}

	#[test]
	fn w() {
		let temp_dir = assert_fs::TempDir::new().unwrap();
		let config_file = temp_dir.child("test.json");
		config_file
			.write_str(r#"{"windows": [{"w": 100}]}"#)
			.unwrap();
		let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
		let output = cmd
			.args(&[
				"-f",
				config_file.path().to_str().unwrap(),
				"config",
				"windows.0.w",
			])
			.output()
			.expect("failed to get command output");
		cmd.assert().success();
		let stdout = String::from_utf8(output.stdout).unwrap();
		assert_eq!("100\n", stdout);
		temp_dir.close().unwrap();
	}

	#[test]
	fn h() {
		let temp_dir = assert_fs::TempDir::new().unwrap();
		let config_file = temp_dir.child("test.json");
		config_file
			.write_str(r#"{"windows": [{"h": 100}]}"#)
			.unwrap();
		let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
		let output = cmd
			.args(&[
				"-f",
				config_file.path().to_str().unwrap(),
				"config",
				"windows.0.h",
			])
			.output()
			.expect("failed to get command output");
		cmd.assert().success();
		let stdout = String::from_utf8(output.stdout).unwrap();
		assert_eq!("100\n", stdout);
		temp_dir.close().unwrap();
	}
}

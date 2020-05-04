// #[macro_use]
// extern crate derive_builder;
use serde::{Deserialize, Serialize};

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

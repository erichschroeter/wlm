#[macro_use]
extern crate derive_builder;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate regex;

#[cfg(windows)]
#[path = "platform/mod.rs"]
pub mod platform;
#[cfg(windows)]
use platform as sys;

pub mod config;
pub mod error;
pub mod window;

use config::Config;

pub const MAX_WINDOW_TITLE_LENGTH: usize = 128;

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

	pub fn monitors(&self) -> Vec<sys::Monitor> {
		sys::list_monitors()
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
}

#[macro_use]
extern crate derive_builder;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate regex;

pub mod config;
pub mod error;
pub mod monitor;
pub mod platform;
pub mod window;

use crate::{
	config::{Config, Window},
	error::Result,
	monitor::Monitor,
};

/// Maximum supported length for a window title.
pub const MAX_WINDOW_TITLE_LENGTH: usize = 128;

/// Returns a list of available monitors.
pub fn monitors() -> Result<Vec<Monitor>> {
	platform::list_monitors()
}

/// Returns a list of windows.
pub fn windows(config: Option<&Config>) -> Result<Vec<Window>> {
	platform::list_windows(config)
}

/// Repositions and resizes displayable windows based on the given config.
pub fn layout(config: &Config) {
	platform::layout_windows(Some(config))
}

pub(crate) fn get_position_string<T: std::fmt::Display>(x: Option<T>, y: Option<T>) -> String {
	match (x, y) {
		(Some(x), Some(y)) => format!("({}, {})", x, y),
		(None, Some(y)) => format!("(null, {})", y),
		(Some(x), None) => format!("({}, null)", x),
		_ => "".to_string(),
	}
}

pub(crate) fn get_dimensions_string<T: std::fmt::Display>(
	width: Option<T>,
	height: Option<T>,
) -> String {
	match (width, height) {
		(Some(w), Some(h)) => format!("{} x {}", w, h),
		(None, Some(h)) => format!("null x {}", h),
		(Some(w), None) => format!("{} x null", w),
		_ => "".to_string(),
	}
}

pub(crate) fn shrink(the_string: &str, shrink_len: usize) -> String {
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
		fn prefix_smaller_than_suffix_when_given_odd_length_string_and_even_shrink_length() {
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

	mod get_position_string {
		use super::super::*;

		#[test]
		fn x_prints_as_null() {
			assert_eq!("(null, 100)", get_position_string(None, Some(100)));
		}

		#[test]
		fn y_prints_as_null() {
			assert_eq!("(100, null)", get_position_string(Some(100), None));
		}

		#[test]
		fn nothing_printed_when_x_and_y_are_null() {
			assert_eq!("", get_position_string(None as Option<&str>, None));
		}
	}

	mod get_dimensions_string {
		use super::super::*;

		#[test]
		fn width_prints_as_null() {
			assert_eq!("null x 100", get_dimensions_string(None, Some(100)));
		}

		#[test]
		fn height_prints_as_null() {
			assert_eq!("100 x null", get_dimensions_string(Some(100), None));
		}

		#[test]
		fn nothing_printed_when_w_and_h_are_null() {
			assert_eq!("", get_dimensions_string(None as Option<&str>, None));
		}
	}
}

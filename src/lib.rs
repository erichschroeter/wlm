#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate serde;

pub mod layout;

#[cfg(windows)]
#[path = "platform/mod.rs"]
pub mod platform;

#[cfg(unix)]
#[path = "platform/mod.rs"]
pub mod platform;

static ELLIPSIS: &str = "...";

#[derive(Debug, Clone, PartialEq)]
pub struct Point {
	pub x: i32,
	pub y: i32,
}

impl Point {
	pub fn new(x: i32, y: i32) -> Self {
		Point { x, y }
	}
}

impl std::fmt::Display for Point {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "({}, {})", self.x, self.y)
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dimensions {
	pub width: i32,
	pub height: i32,
}

impl Dimensions {
	pub fn new(width: i32, height: i32) -> Self {
		Dimensions { width, height }
	}
}

impl std::fmt::Display for Dimensions {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}x{}", self.width, self.height)
	}
}

/// Shrinks a string to a specified maximum length by keeping the left portion
/// and replacing the right portion with an ellipsis ("...").
///
/// # Arguments
///
/// * `s` - The string to shrink.
/// * `max_length` - The maximum length of the string after shrinking. If `max_length`
///   is less than or equal to the length of `s`, the original string is returned.
///   If `max_length` is smaller than the length of the ellipsis, the function
///   will return the ellipsis only.
///
/// # Returns
///
/// A new string that is either the original string (if its length is less than
/// or equal to `max_length`) or a shrunk version with the right part replaced
/// by an ellipsis.
///
/// # Examples
///
/// ```
/// # use wlm::shrink_right;
/// let example = "Hello, World!";
/// let shrunk = shrink_right(example, 8);
/// assert_eq!(shrunk, "Hello...");
///
/// let short_example = "Hi";
/// let not_shrunk = shrink_right(short_example, 10);
/// assert_eq!(not_shrunk, "Hi");
///
/// let edge_case = shrink_right("Hello, World!", 3);
/// assert_eq!(edge_case, "...");
/// ```
#[allow(dead_code)]
pub fn shrink_right(s: &str, max_length: usize) -> String {
	if s.len() <= max_length {
		s.to_string()
	} else if max_length < ELLIPSIS.len() {
		format!("{}", &ELLIPSIS[..max_length])
	} else {
		let effective_length = max_length.saturating_sub(ELLIPSIS.len());
		format!("{}{}", &s[..effective_length], ELLIPSIS)
	}
}

#[cfg(test)]
mod test_shrink_right {
	use super::*;

	#[test]
	fn string_less_than_ellipsis() {
		let input = "He";
		let output = shrink_right(input, 2);
		assert_eq!("He", output);
	}

	#[test]
	fn string_same_as_ellipsis() {
		let input = "Her";
		let output = shrink_right(input, 3);
		assert_eq!("Her", output);
	}

	#[test]
	fn long_string_with_max_same_as_ellipsis() {
		let input = "Hello, World!";
		let output = shrink_right(input, 3);
		assert_eq!("...", output);
	}

	#[test]
	fn long_string_with_max_less_than_ellipsis() {
		let input = "Hello, World!";
		let output = shrink_right(input, 1);
		assert_eq!(".", output);
	}

	#[test]
	fn string_less_than_max() {
		let input = "Hello, World!";
		let output = shrink_right(input, 20);
		assert_eq!("Hello, World!", output);
	}

	#[test]
	fn string_more_than_max() {
		let input = "Hello, World!";
		let output = shrink_right(input, 6);
		assert_eq!("Hel...", output);
	}

	#[test]
	fn string_same_as_max() {
		let input = "Hello, World!";
		let output = shrink_right(input, input.len());
		assert_eq!("Hello, World!", output);
	}

	#[test]
	fn empty_string() {
		let input = "";
		let output = shrink_right(input, 5);
		assert_eq!("", output);
	}

	#[test]
	fn to_zero_length() {
		let input = "Hello, World!";
		let output = shrink_right(input, 0);
		assert_eq!("", output);
	}
}

/// Shrinks a string to a specified maximum length by keeping the right portion
/// and replacing the left portion with an ellipsis ("...").
///
/// # Arguments
///
/// * `s` - The string to shrink.
/// * `max_length` - The maximum length of the string after shrinking. If `max_length`
///   is less than or equal to the length of `s`, the original string is returned.
///   If `max_length` is smaller than the length of the ellipsis, the function
///   will return the ellipsis only.
///
/// # Returns
///
/// A new string that is either the original string (if its length is less than
/// or equal to `max_length`) or a shrunk version with the left part replaced
/// by an ellipsis.
///
/// # Examples
///
/// ```
/// # use wlm::shrink_left;
/// let example = "Hello, World!";
/// let shrunk = shrink_left(example, 8);
/// assert_eq!(shrunk, "...orld!");
///
/// let short_example = "Hi";
/// let not_shrunk = shrink_left(short_example, 10);
/// assert_eq!(not_shrunk, "Hi");
///
/// let edge_case = shrink_left("Hello, World!", 3);
/// assert_eq!(edge_case, "...");
/// ```
#[allow(dead_code)]
pub fn shrink_left(s: &str, max_length: usize) -> String {
	if s.len() <= max_length {
		s.to_string()
	} else if max_length < ELLIPSIS.len() {
		format!("{}", &ELLIPSIS[..max_length])
	} else {
		let effective_length = max_length.saturating_sub(ELLIPSIS.len());
		format!("{}{}", ELLIPSIS, &s[s.len() - effective_length..])
	}
}

#[cfg(test)]
mod test_shrink_left {
	use super::*;

	#[test]
	fn string_less_than_ellipsis() {
		let input = "He";
		let output = shrink_left(input, 2);
		assert_eq!("He", output);
	}

	#[test]
	fn string_same_as_ellipsis() {
		let input = "Her";
		let output = shrink_left(input, 3);
		assert_eq!("Her", output);
	}

	#[test]
	fn long_string_with_max_same_as_ellipsis() {
		let input = "Hello, World!";
		let output = shrink_left(input, 3);
		assert_eq!("...", output);
	}

	#[test]
	fn long_string_with_max_less_than_ellipsis() {
		let input = "Hello, World!";
		let output = shrink_left(input, 1);
		assert_eq!(".", output);
	}

	#[test]
	fn string_less_than_max() {
		let input = "Hello, World!";
		let output = shrink_left(input, 20);
		assert_eq!("Hello, World!", output);
	}

	#[test]
	fn string_more_than_max() {
		let input = "Hello, World!";
		let output = shrink_left(input, 6);
		assert_eq!("...ld!", output);
	}

	#[test]
	fn string_same_as_max() {
		let input = "Hello, World!";
		let output = shrink_left(input, input.len());
		assert_eq!("Hello, World!", output);
	}

	#[test]
	fn empty_string() {
		let input = "";
		let output = shrink_left(input, 5);
		assert_eq!("", output);
	}

	#[test]
	fn to_zero_length() {
		let input = "Hello, World!";
		let output = shrink_left(input, 0);
		assert_eq!("", output);
	}
}

/// Shrinks a string to a specified length by keeping equal parts from the start and end
/// and replacing the middle part with an ellipsis ("..."). If the `max_length` is odd, one extra
/// character is taken from the beginning.
///
/// # Arguments
///
/// * `s` - The string to shrink.
/// * `max_length` - The desired length of the string after shrinking, including the ellipsis.
///   If `max_length` is less than or equal to the length of `s`, the original string is returned.
///   If `max_length` is smaller than the length of the ellipsis, the function will return the
///   ellipsis only.
///
/// # Returns
///
/// A new string that is either the original string (if its length is less than or equal to
/// `max_length`) or a shrunk version with the middle part replaced by an ellipsis.
///
/// # Examples
///
/// ```
/// # use wlm::shrink_center;
/// let example = "Hello, World!";
/// let shrunk = shrink_center(example, 7);
/// assert_eq!(shrunk, "He...d!");
/// let shrunk = shrink_center(example, 8);
/// assert_eq!(shrunk, "He...ld!");
///
/// let short_example = "Hi";
/// let not_shrunk = shrink_center(short_example, 10);
/// assert_eq!(not_shrunk, "Hi");
///
/// let edge_case = shrink_center("Hello, World!", 3);
/// assert_eq!(edge_case, "...");
/// ```
#[allow(dead_code)]
pub fn shrink_center(s: &str, max_length: usize) -> String {
	let len = s.len();
	if len <= max_length {
		return s.to_string();
	} else if max_length < ELLIPSIS.len() {
		return format!("{}", &ELLIPSIS[..max_length]);
	}

	// Adjust for the 3 characters in "..."
	let effective_length = max_length.saturating_sub(ELLIPSIS.len());
	let half_count = effective_length / 2;
	let start = &s[..half_count];
	let end = if effective_length % 2 == 0 {
		&s[len - half_count..]
	} else {
		&s[len - half_count - 1..]
	};

	format!("{}{}{}", start, ELLIPSIS, end)
}

#[cfg(test)]
mod test_shrink_center {
	use super::*;

	#[test]
	fn string_less_than_ellipsis() {
		let input = "He";
		let output = shrink_center(input, 2);
		assert_eq!("He", output);
	}

	#[test]
	fn string_same_as_ellipsis() {
		let input = "Her";
		let output = shrink_center(input, 3);
		assert_eq!("Her", output);
	}

	#[test]
	fn long_string_with_max_same_as_ellipsis() {
		let input = "Hello, World!";
		let output = shrink_center(input, 3);
		assert_eq!("...", output);
	}

	#[test]
	fn long_string_with_max_less_than_ellipsis() {
		let input = "Hello, World!";
		let output = shrink_center(input, 1);
		assert_eq!(".", output);
	}

	#[test]
	fn string_less_than_max() {
		let input = "Hello, World!";
		let output = shrink_center(input, 20);
		assert_eq!("Hello, World!", output);
	}

	#[test]
	fn string_more_than_even_max() {
		let input = "Hello, World!";
		let output = shrink_center(input, 6);
		assert_eq!("H...d!", output);
	}

	#[test]
	fn string_more_than_odd_max() {
		let input = "Hello, World!";
		let output = shrink_center(input, 7);
		assert_eq!("He...d!", output);
	}

	#[test]
	fn string_same_as_max() {
		let input = "Hello, World!";
		let output = shrink_center(input, input.len());
		assert_eq!("Hello, World!", output);
	}

	#[test]
	fn empty_string() {
		let input = "";
		let output = shrink_center(input, 5);
		assert_eq!("", output);
	}

	#[test]
	fn to_zero_length() {
		let input = "Hello, World!";
		let output = shrink_center(input, 0);
		assert_eq!("", output);
	}
}

/// A trait for managing window layouts on a screen.
///
/// Implementors of this trait are responsible for providing a collection of windows
/// and a method to layout these windows based on a given configuration.
pub trait WindowProvider {
	/// Returns a vector of `Screen` instances with `Window` instances.
	///
	/// This method should be implemented to provide the current set of screens
	/// along with the windows that the provider is managing.
	///
	/// # Examples
	///
	/// ```
	/// # use wlm::{default_window_provider, WindowProvider};
	/// let provider = default_window_provider();
	/// let screens = provider.screens();
	/// for screen in screens {
	///     println!("{}", screen);
	/// 	for window in screen.windows {
	///  	   println!("{}", window);
	///	 	}
	/// }
	/// ```
	fn screens(&self) -> Vec<layout::Screen>;

	/// Lays out windows based on the specified configuration.
	///
	/// This method should be implemented to arrange the windows according
	/// to the provided configuration.
	///
	/// # Arguments
	///
	/// * `config` - A reference to a `Layout` instance that specifies
	///   the layout configuration.
	///
	/// # Examples
	///
	/// ```
	/// # use wlm::{default_window_provider, layout::Layout, WindowProvider};
	/// let provider = default_window_provider();
	/// let config = Layout::new();
	/// provider.layout(&config);
	/// ```
	fn layout(&self, config: &layout::Layout);
}

/// Provides a default window provider.
///
/// This function is a factory method that creates an instance of a type
/// that implements the `WindowProvider` trait. The implementation varies
/// based on the target operating system.
///
/// # Examples
///
/// ```
/// use wlm::{default_window_provider, WindowProvider, layout::Layout};
/// let provider = default_window_provider();
/// let windows = provider.screens();
/// provider.layout(&Layout::new(/* ... */));
/// ```
///
/// # Platform-specific Behavior
///
/// - On Windows platforms, this will return a `Win32Provider`.
pub fn default_window_provider() -> impl WindowProvider {
	#[cfg(windows)]
	let provider = crate::platform::win::Win32Provider::default();
	#[cfg(unix)]
	let provider = crate::platform::unix::X11Provider::default();
	provider
}

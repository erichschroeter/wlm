pub static FORMAT_NAMES: [&str; 2] = ["table", "yaml"];
pub const MAX_WINDOW_TITLE_LENGTH: usize = 128;

#[derive(Debug, Clone, Copy)]
pub enum Format {
	Table,
	Yaml,
}

impl Format {
	/// Returns the string representation of the `Format`.
	///
	/// This returns the same string as the `fmt::Display` implementation.
	pub fn as_str(&self) -> &'static str {
		FORMAT_NAMES[*self as usize]
	}
}

impl std::str::FromStr for Format {
	type Err = String;

	fn from_str(format: &str) -> Result<Format, Self::Err> {
		match format {
			"table" => Ok(Format::Table),
			"yaml" => Ok(Format::Yaml),
			_ => Err(format!("Failed to parse into string '{format}'")),
		}
	}
}

impl std::fmt::Display for Format {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		fmt.pad(self.as_str())
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Builder)]
#[builder(setter(into))]
pub struct Layout {
	#[builder(default)]
	pub screens: Vec<Screen>,
}

impl Layout {
	pub fn new() -> Self {
		Self {
			screens: Vec::new(),
		}
	}
}

impl Default for Layout {
	fn default() -> Self {
		Layout::new()
	}
}

/// The `Window` struct represents the configuration of a window in a window manager.
///
/// This struct allows for optional customization of various window properties such as size, coordinates,
/// process name, and window state (maximized, minimized, etc.). Each field is optional, allowing for
/// flexibility in specifying only the desired attributes.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Builder)]
#[builder(setter(into))]
pub struct Window {
	/// The title of the window. This is an optional field, and if not provided, a default
	/// value may be used depending on the context.
	#[builder(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title: Option<String>,

	/// The process associated with the window, typically represented as a string.
	/// This is optional and can be omitted if not applicable.
	#[builder(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub process: Option<String>,

	/// The top left x-coordinate of the window's position.
	#[builder(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub x: Option<String>,

	/// The top left y-coordinate of the window's position.
	#[builder(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub y: Option<String>,

	/// The z-order of the window, which determines its stacking order relative to other windows.
	#[builder(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub z: Option<i32>,

	/// The width of the window.
	///
	/// # Examples
	/// ### Using pixels
	/// ```yaml
	/// screens:
	///   windows:
	///   - title: 'Some title'
	///     w: '940'
	/// ```
	/// ### Using percentage
	/// ```yaml
	/// screens:
	///   windows:
	///   - title: 'Some title'
	///     w: '40%'
	/// ```
	#[builder(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub w: Option<String>,

	/// The height of the window. Similar to width, this is optional and a default value
	/// may be used if not provided.
	#[builder(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub h: Option<String>,

	/// A flag indicating whether the window is maximized. This is optional and defaults to
	/// `false` if not specified.
	#[builder(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub maximized: Option<bool>,

	/// A flag indicating whether the window is maximized vertically. Optional and defaults
	/// to `false` if not specified.
	#[builder(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub maximized_vertical: Option<bool>,

	/// A flag indicating whether the window is maximized horizontally. Optional and defaults
	/// to `false` if not specified.
	#[builder(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub maximized_horizontal: Option<bool>,

	/// A flag indicating whether the window is minimized. This is optional and defaults to
	/// `false` if not specified.
	#[builder(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub minimized: Option<bool>,
}

impl Window {
	pub fn new() -> Self {
		Window::default()
	}
}

impl std::fmt::Display for Window {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Builder)]
#[builder(setter(into))]
pub struct Screen {
	#[builder(default)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub id: Option<u8>,
	#[builder(default)]
	pub windows: Vec<Window>,
}

impl Screen {
	pub fn new() -> Self {
		Screen {
			id: None,
			windows: Vec::new(),
		}
	}
}

impl std::fmt::Display for Screen {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

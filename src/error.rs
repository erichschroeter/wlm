#[derive(Fail, Debug)]
pub enum Error {
	#[fail(display = "IO error: {}", error)]
	Io { error: std::io::Error },
	#[fail(display = "Validation error: {}", error)]
	Validation { error: serde_json::Error },
	/// A property value is invalid.
	#[fail(display = "Invalid property")]
	InvalidProperty,
	/// An index value is invalid.
	#[fail(display = "Invalid index")]
	InvalidIndex,
}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Error {
		Error::Io { error: err }
	}
}

impl From<serde_json::Error> for Error {
	fn from(err: serde_json::Error) -> Error {
		Error::Validation { error: err }
	}
}

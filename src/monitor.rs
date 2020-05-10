#[derive(Debug, Clone)]
pub struct Monitor {
	pub name: String,
	pub position: (i32, i32),
	pub size: (i32, i32),
}

impl Monitor {
	pub fn new() -> Self {
		Monitor {
			position: (0, 0),
			size: (0, 0),
			name: "".to_string(),
		}
	}
}

impl std::fmt::Display for Monitor {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:#?}", &self)
	}
}

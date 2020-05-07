
pub trait Window {
	fn title(&self) -> String;
	fn process(&self) -> String;
}

pub trait Position {
	fn x(&self) -> i32;
	fn y(&self) -> i32;
}

pub trait Dimension {
	fn width(&self) -> u32;
	fn height(&self) -> u32;
}

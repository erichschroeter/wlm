pub trait Window {
	fn title(&self) -> String;
	fn process(&self) -> String;
	fn position(&self) -> dyn Position;
	fn dimension(&self) -> dyn Dimension;
}

pub trait Position {
	fn x(&self) -> i32;
	fn y(&self) -> i32;
}

pub trait PositionBuilder {
	fn with_x(&mut self, value: i32) -> &mut Self;
	fn with_y(&mut self, value: i32) -> &mut Self;
}

pub trait Dimension {
	fn width(&self) -> u32;
	fn height(&self) -> u32;
}

pub trait DimensionBuilder {
	fn with_width(&self, value: u32) -> &mut Self;
	fn with_height(&self, value: u32) -> &mut Self;
}

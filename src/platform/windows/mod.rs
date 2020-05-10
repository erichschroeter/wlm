use winapi::shared::windef::RECT;

pub mod monitor;
pub mod window;

/// Returns the top left point from the given [RECT](https://docs.microsoft.com/en-us/windows/win32/api/windef/ns-windef-rect).
pub fn get_position(rect: RECT) -> (i32, i32) {
	(rect.left, rect.top)
}

/// Returns the (width, height) from the given [RECT](https://docs.microsoft.com/en-us/windows/win32/api/windef/ns-windef-rect).
pub fn get_dimensions(rect: RECT) -> (i32, i32) {
	(
		(rect.right - rect.left).abs(),
		(rect.bottom - rect.top).abs(),
	)
}

#[cfg(test)]
mod tests {
	mod get_position {
		use super::super::*;

		#[test]
		fn nominal_case() {
			let rect = RECT {
				left: 0,
				top: 1,
				right: 2,
				bottom: 3,
			};
			assert_eq!((0, 1), get_position(rect));
		}
	}

	mod get_dimensions {
		use super::super::*;

		#[test]
		fn zero_width_given_same_left_and_right() {
			let rect = RECT {
				left: 100,
				top: 0,
				right: 100,
				bottom: 0,
			};
			assert_eq!((0, 0), get_dimensions(rect));
		}

		#[test]
		fn zero_height_given_same_top_and_bottom() {
			let rect = RECT {
				left: 0,
				top: 100,
				right: 0,
				bottom: 100,
			};
			assert_eq!((0, 0), get_dimensions(rect));
		}

		#[test]
		fn positive_width_given_left_greater_than_right() {
			let rect = RECT {
				left: 100,
				top: 0,
				right: 20,
				bottom: 0,
			};
			assert_eq!((80, 0), get_dimensions(rect));
		}

		#[test]
		fn positive_height_given_top_greater_than_bottom() {
			let rect = RECT {
				left: 0,
				top: 100,
				right: 0,
				bottom: 20,
			};
			assert_eq!((0, 80), get_dimensions(rect));
		}
	}
}

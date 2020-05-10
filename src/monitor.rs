use crate::{get_dimensions_string, get_position_string};
use prettytable::{color, format, Attr, Cell, Row, Table};

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

fn get_monitor_table(monitors: &[Monitor]) -> Table {
	let mut table = Table::new();
	table.set_format(*format::consts::FORMAT_NO_COLSEP);
	table.add_row(Row::new(vec![
		Cell::new("Name").style_spec("c"),
		Cell::new("Position").style_spec("c"),
		Cell::new("Dimension").style_spec("c"),
	]));
	for m in monitors {
		let mut row = Row::empty();
		if !m.name.is_empty() {
			row.add_cell(Cell::new(&m.name).with_style(Attr::ForegroundColor(color::RED)));
		} else {
			row.add_cell(Cell::default());
		}
		row.add_cell(
			Cell::new(&get_position_string(Some(m.position.0), Some(m.position.1)))
				.with_style(Attr::ForegroundColor(color::YELLOW)),
		);
		row.add_cell(Cell::new(&get_dimensions_string(
			Some(m.size.0),
			Some(m.size.1),
		)));
		table.add_row(row);
	}
	table
}

pub fn print_monitors<T>(monitors: &[Monitor], out: &mut T)
where
	T: std::io::Write + ?Sized,
{
	let _ = get_monitor_table(monitors).print(out);
}

pub fn print_monitors_tty(monitors: &[Monitor]) {
	let _ = get_monitor_table(monitors).printstd();
}

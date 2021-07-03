mod rust;

pub use json::JsonValue;
pub use rust::RustLexer;
pub use crossterm::style::Color;
use crate::file::Row;

#[derive(Debug)]
pub struct Parsed {
  original: String,
  range: std::ops::Range<usize>,
  color: Option<Color>
}

impl Parsed {
  pub fn get_original(&self) -> &str {
    &self.original
  }

  pub fn get_color(&self) -> Option<&Color> {
    self.color.as_ref()
  }
}

pub trait Lexer<'a> {
  fn highlight_off() -> Self;
  fn lex(rows: &'a Vec<Row>) -> Self;
  fn parse(&self) -> Option<Vec<Vec<Parsed>>>;
}

fn get_color(color_str: &str) -> Option<Color> {
  match color_str {
    "blue" => Some(Color::Blue),
    "darkblue" => Some(Color::DarkBlue),
    "red" => Some(Color::Red),
    "purple" => Some(Color::Magenta),
    "green" => Some(Color::Green),
    "yellow" => Some(Color::Yellow),
    "orange" => Some(Color::DarkYellow),
    "grey" => Some(Color::DarkGrey),
    _ => None
  }
}

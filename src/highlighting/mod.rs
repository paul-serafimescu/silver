mod rust;
mod python;

pub use json::JsonValue;
pub use rust::RustLexer;
pub use python::PythonLexer;
pub use crossterm::style::{Color, Attribute};
use crate::file::Row;

#[derive(Debug)]
pub struct Parsed {
  original: String,
  range: std::ops::Range<usize>,
  color: Option<Color>,
  attr: Attribute
}

impl Parsed {
  pub fn get_original(&self) -> &str {
    &self.original
  }

  pub fn get_color_and_attribute(&self) -> (Option<&Color>, &Attribute) {
    (self.color.as_ref(), &self.attr)
  }
}

pub trait Lexer<'a> {
  fn highlight_off() -> Self;
  fn lex(rows: &'a Vec<Row>, syntax_file: Option<&JsonValue>) -> Self;
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

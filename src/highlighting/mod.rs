#![allow(dead_code)]
mod rust;

pub use json::JsonValue;
pub use rust::RustLexer;
pub use crossterm::style::Color;
use crate::file::Row;

#[derive(Debug)]
pub enum Token {
  Str,
  Number,
  Type,
  Function, // I won't bother with this one for now
  Keyword,
  Char,
  Unknown
}

#[derive(Debug)]
pub struct Parsed {
  original: String,
  parsed: Token,
  color: Option<Color>
}

impl Parsed {
  pub fn get_original(&self) -> &str {
    &self.original
  }

  pub fn get_parsed(&self) -> &Token {
    &self.parsed
  }

  pub fn get_color(&self) -> Option<&Color> {
    self.color.as_ref()
  }
}

pub trait Lexer {
  fn default() -> Self;
  fn lex(&self, rows: &Vec<Row>) -> Option<Vec<Vec<Parsed>>>;
  fn parse(token: &str, syntax_rules: &JsonValue) -> Parsed;
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
    _ => None
  }
}

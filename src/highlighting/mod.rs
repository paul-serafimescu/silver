#![allow(dead_code)]
mod rust;

pub use rust::RustLexer;
pub use crossterm::style::Color;
use super::file::Row;

pub enum Token {
  String,
  Number,
  Function,
  Keyword,
  None
}

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
}

pub trait Lexer {
  fn lex(rows: &Vec<Row>) -> Vec<Vec<Parsed>>;
  fn parse(token: &str) -> Parsed;
}

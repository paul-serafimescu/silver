#![allow(unused_imports)]
use crate::highlighting::{
  Lexer, Parsed, Token, Row, Color
};

pub struct RustLexer;

impl Lexer for RustLexer {
  fn lex(rows: &Vec<Row>) -> Vec<Vec<Parsed>> {
    let mut parsed = Vec::new();
    for row in rows {
      let mut parsed_row = Vec::new();
      let split_row = row.content()
        .clone()
        .split_whitespace()
        .collect::<Vec<&str>>();
      for token in split_row {
        parsed_row.push(RustLexer::parse(token));
      }
      parsed.push(parsed_row)
    }
    parsed
  }

  // TODO: use syntax/rust.json to encode tokens & respective colors
  fn parse(token: &str) -> Parsed {
    Parsed {
      original: String::from(token),
      parsed: Token::None,
      color: None
    }
  }
}

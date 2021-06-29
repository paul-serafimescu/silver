#![allow(unused_imports)]
use json::JsonValue;
use std::fs;
use std::env;
use std::io::Read;
use crate::highlighting::{
  Lexer, Parsed, Token, Row, Color,
  get_color
};

pub struct RustLexer {
  _syntax: Option<json::JsonValue>
}

impl Lexer for RustLexer {
  fn default() -> Self {
    let path = env::current_dir().unwrap().join("syntax/rust.json");
    if let Ok(file_contents) = fs::read_to_string(path) {
      Self {
        _syntax: if let Ok(result) = json::parse(&file_contents) {
          Some(result)
        } else { None }
      }
    } else {
      Self {
        _syntax: None
      }
    }
  }

  fn lex(&self, rows: &Vec<Row>) -> Option<Vec<Vec<Parsed>>> {
    if let Some(syntax_rules) = &self._syntax {
      let mut parsed = Vec::new();
      for row in rows {
        let mut parsed_row = Vec::new();
        let split_row = row.content()
          .clone()
          .split_whitespace()
          .collect::<Vec<&str>>();
        for token in split_row {
          parsed_row.push(RustLexer::parse(token, &syntax_rules));
        }
        parsed.push(parsed_row)
      }
      Some(parsed)
    } else {
      None
    }
  }

  // TODO: use syntax/rust.json to encode tokens & respective colors
  fn parse(token: &str, syntax_rules: &JsonValue) -> Parsed {
    let (parsed, color) = match_token(token, syntax_rules);
    Parsed {
      original: String::from(token),
      parsed,
      color
    }
  }
}

fn type_of(token: &str, syntax_rules: &JsonValue) -> Token {
  let keywords = &syntax_rules["keywords"];
  let types = &syntax_rules["types"];
  if keywords.contains(token) {
    Token::Keyword
  } else if types.contains(token) {
    Token::Type
  } else if token.parse::<f64>().is_ok() {
    Token::Number
  } else if token.starts_with('\'') && token.ends_with('\'') {
    Token::Char
  } else if token.starts_with('\"') && token.ends_with('\"') {
    Token::Str
  } else {
    Token::Unknown
  }
}

fn match_token(token: &str, syntax_rules: &JsonValue) -> (Token, Option<Color>) {
  let token_type = type_of(token, syntax_rules);
  let color = match &token_type {
    Token::Keyword => get_color(&syntax_rules["colors"]["keyword"].as_str().unwrap()),
    Token::Type => get_color(&syntax_rules["colors"]["type"].as_str().unwrap()),
    Token::Number => get_color(&syntax_rules["colors"]["number"].as_str().unwrap()),
    Token::Char => get_color(&syntax_rules["colors"]["char"].as_str().unwrap()),
    Token::Str => get_color(&syntax_rules["colors"]["string"].as_str().unwrap()),
    _ => None
  };
  (token_type, color)
}

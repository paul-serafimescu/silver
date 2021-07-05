// why did I do this? because python doesn't deserve meaningful syntax highlighting
use std::ops::Range;
use rand::Rng;
use unicode_segmentation::UnicodeSegmentation;
use crate::highlighting::{
  Lexer, Parsed, Row, Color, get_color, JsonValue
};

#[derive(Debug, PartialEq)]
enum PythonToken {
  DontCare
}

pub struct PythonLexer<'a> {
  _lexed: Option<Vec<Vec<(PythonToken, Range<usize>)>>>,
  _raw: Option<&'a Vec<Row>>,
}

impl<'a> PythonLexer<'a> {
  fn lexer(content: &str) -> Vec<(PythonToken, Range<usize>)> {
    let mut lexed = Vec::new();
    let length = content.graphemes(true).count();
    if length < 3 {
      lexed.push((PythonToken::DontCare, Range {
        start: 0,
        end: length
      }));
      return lexed
    }
    let (mut start, mut end) = (0, 0);
    while end < length {
      let mut rng = rand::thread_rng();
      let rnd = rng.gen_range::<usize, Range<usize>>(0..(length + 1));
      let addition = std::cmp::min(length - end, rnd / 3);
      end += addition;
      lexed.push((PythonToken::DontCare, Range {
        start,
        end
      }));
      start += addition;
    }
    lexed
  }
}

impl<'a> Lexer<'a> for PythonLexer<'a> {

  fn highlight_off() -> Self {
    Self {
      _lexed: None,
      _raw: None,
    }
  }

  fn lex(rows: &'a Vec<Row>, syntax_file: Option<&JsonValue>) -> Self {
    if let None = syntax_file {
      return Self::highlight_off()
    }
    let mut lex = Vec::new();
    for row in rows {
      let lexed = PythonLexer::lexer(row.content());
      lex.push(lexed)
    }
    Self {
      _raw: Some(rows),
      _lexed: Some(lex)
    }
  }

  fn parse(&self) -> Option<Vec<Vec<Parsed>>> {
    if self._lexed == None {
      return None
    }
    let lexed = self._lexed.as_ref().unwrap();
    let mut parsed_file = Vec::new();
    let mut raw_content_iter = self._raw.unwrap().into_iter();
    for row in lexed {
      let raw_row = raw_content_iter.next().unwrap();
      let mut parsed_row = Vec::new();
      for (_, range) in row {
        let original = String::from(raw_row.content()[range.clone()].to_string());
        parsed_row.push(Parsed {
          color: match_color(),
          range: range.clone(),
          original
        })
      }
      parsed_file.push(parsed_row)
    }
    Some(parsed_file)
  }
}

fn match_color() -> Option<Color> {
  let mut rng = rand::thread_rng();
  match rng.gen::<u32>() % 7 {
    0 => None,
    1 => get_color("green"),
    2 => get_color("yellow"),
    3 => get_color("darkblue"),
    4 => None,
    5 => get_color("orange"),
     _ => None
  }
}

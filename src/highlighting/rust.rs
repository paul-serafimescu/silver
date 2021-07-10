use json::JsonValue;
use crate::highlighting::{
  Lexer, Parsed, Row, Color,
  get_color, Attribute
};
use logos::{Logos, Lexer as LogosLexer};

fn trim_function(token: &mut LogosLexer<RustToken>) -> String {
  let mut string = token.slice().to_string();
  string.pop();
  string
}

#[derive(Debug, Logos, PartialEq)]
enum RustToken {
  #[regex("\"([^\"]*)\"", priority=100)]
  String,

  #[regex("\'([^\']*)\'")]
  Char,

  #[regex(r"-?[0-9]+(\.[0-9]+)?")]
  Number,

  #[regex(r"([a-zA-Z]+_?)*!?\(", trim_function)]
  Function(String),

  #[token(" as ")]
  #[regex("\\s?fn\\s")]
  #[regex("\\s?impl\\s")]
  #[regex("\\s?for\\s")]
  #[regex("\\s?in\\s")]
  #[regex("\\s?use\\s")]
  #[regex("\\s?mod\\s")]
  #[regex("\\s?trait\\s")]
  #[regex("\\s?pub\\s")]
  #[regex("(&?|\\s?)mut\\s")]
  #[regex("\\s?enum\\s")]
  #[regex("\\s?let\\s")]
  #[regex("\\s?const\\s")]
  #[regex("\\s?true(\\s?|;)")]
  #[regex("\\s?false(\\s?|;)")]
  #[regex("\\s?break(\\s?|;)")]
  #[regex("\\s?continue(\\s?|;)")]
  #[regex("\\s?if\\s")]
  #[regex("\\s?else\\s")]
  #[token("struct ")]
  #[token("macro_rules! ")]
  #[token("match ")]
  #[token("dyn ")]
  #[token("loop")]
  Keyword,

  #[regex("(u|i)(8|16|32|64|128)")]
  #[token("self")]
  #[token("Self")]
  #[token("Vec")]
  #[token("Option")]
  #[token("Result")]
  #[token("Ok")]
  #[token("Box")]
  #[token("String")]
  #[token("&str")]
  #[token("None")]
  #[token("usize")]
  #[token("char")]
  Type,

  #[regex("//.+", priority = 100)]
  Comment,

  #[regex("[ \\t\\n\\r\\f\\v]+")]
  #[error]
  DontCare
}

pub struct RustLexer<'a> {
  _syntax: Option<json::JsonValue>,
  _lex: Option<Vec<Vec<(RustToken, std::ops::Range<usize>)>>>,
  _raw: Option<&'a Vec<Row>>
}

impl<'a> Lexer<'a> for RustLexer<'a> {

  fn highlight_off() -> Self {
    Self {
      _syntax: None,
      _lex: None,
      _raw: None
    }
  }

  fn lex(rows: &'a Vec<Row>, syntax_file: Option<&JsonValue>) -> Self {
    if let Some(syntax) = syntax_file {
      let mut lex = Vec::new();
      for row in rows {
        let mut row_lex = Vec::new();
        let lexed = RustToken::lexer(row.content()).spanned();
        for token_range in lexed {
          match token_range.0 {
            RustToken::Function(name) => {
              row_lex.push((RustToken::Function(name), std::ops::Range {
                start: token_range.1.start,
                end: token_range.1.end - 1
              }));
              row_lex.push((RustToken::DontCare, std::ops::Range {
                start: token_range.1.end - 1,
                end: token_range.1.end
              }))
            },
            _ => row_lex.push(token_range)
          }
        }
        lex.push(row_lex)
      }
      Self {
        _lex: Some(lex),
        _syntax: Some(syntax.clone()),
        _raw: Some(rows)
      }
    } else {
      Self::highlight_off()
    }
  }

  // TODO: use syntax/rust.json to encode tokens & respective colors
  fn parse(&self) -> Option<Vec<Vec<Parsed>>> {
    if self._lex == None {
      return None
    }
    let lexed = self._lex.as_ref().unwrap();
    let mut parsed_file = Vec::new();
    let mut raw_content_iter = self._raw.unwrap().into_iter();
    for row in lexed {
      let raw_row = raw_content_iter.next().unwrap();
      let mut parsed_row = Vec::new();
      for (token, range) in row {
        let original = String::from(raw_row.content()[range.clone()].to_string());
        parsed_row.push(Parsed {
          color: match_color(token, self._syntax.as_ref().unwrap()),
          range: range.clone(),
          original,
          attr: get_attribute(token, self._syntax.as_ref().unwrap())
        })
      }
      parsed_file.push(parsed_row)
    }
    Some(parsed_file)
  }
}

fn match_color(token: &RustToken, syntax_rules: &JsonValue) -> Option<Color> {
  let colors = &syntax_rules["colors"];
  match token {
    RustToken::Keyword => get_color(colors["keyword"].as_str().unwrap()),
    RustToken::Type => get_color(colors["type"].as_str().unwrap()),
    RustToken::Char => get_color(colors["char"].as_str().unwrap()),
    RustToken::String => get_color(colors["string"].as_str().unwrap()),
    RustToken::Comment => get_color(colors["comment"].as_str().unwrap()),
    RustToken::Number => get_color(colors["number"].as_str().unwrap()),
    RustToken::Function(_)  => get_color(colors["function"].as_str().unwrap()),
    _ => None
  }
}

fn get_attribute(token: &RustToken, syntax_rules: &JsonValue) -> Attribute {
  let token_type = match token {
    RustToken::Keyword => "keyword",
    RustToken::Type => "type",
    RustToken::Char => "char",
    RustToken::String => "string",
    RustToken::Comment => "comment",
    RustToken::Number => "number",
    RustToken::Function(_)  => "function",
    _ => ""
  };
  if let Some(attribute) = &syntax_rules["style"][token_type].as_str() {
    match *attribute {
      "bold" => Attribute::Bold,
      "italic" => Attribute::Italic,
      _ => Attribute::NormalIntensity
    }
  } else {
    Attribute::NormalIntensity
  }
}

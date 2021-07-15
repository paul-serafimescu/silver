use crate::highlighting::{
  Lexer, Parsed, Row, Color,
  get_color, Attribute, Logos, LogosLexer,
  JsonValue
};

fn trim_function(token: &mut LogosLexer<CToken>) -> String {
  let mut string = token.slice().to_string();
  string.pop();
  string
}

#[derive(Debug, Logos, PartialEq)]
enum CToken {
  #[regex("\"([^\"]*)\"", priority = 100)]
  String,

  #[regex(r"'([^']*)'")]
  Char,

  #[regex(r" -?[0-9]+(\.[0-9]+)?")]
  Number,

  #[regex(r"([a-zA-Z]+_?)*!?\(", trim_function)]
  Function(String),

  #[token("#include")]
  #[token("#define")]
  #[token("#ifndef")]
  #[token("#endif")]
  #[token("if ")]
  #[token("else ")]
  #[token("while ")]
  #[token("do ")]
  #[token("for ")]
  #[token("enum ")]
  #[token("struct ")]
  #[token("break")]
  #[token("true ")]
  #[token("false ")]
  #[token("continue")]
  #[token("return")]
  #[token("switch")]
  #[token("case")]
  #[token("const")]
  #[token("typedef")]
  #[token("union")]
  #[token("default")]
  Keyword,

  #[regex(r"u?int_(8|16|32|64)_t")]
  #[regex(r"<(([a-zA-Z0-9]|-|_)+/?)+\.h>")]
  #[token("char *")]
  #[token("int")]
  #[token("unsigned")]
  #[token("NULL")]
  #[token("bool")]
  #[token("short")]
  #[token("long")]
  #[token("void")]
  #[token("char")]
  #[token("[]")]
  #[regex(r"[a-zA-Z]+_t")]
  Type,

  #[regex(r"//.+", priority = 100)]
  Comment,

  #[token("/*")]
  MultiLineCommentStart,

  #[token("*/")]
  MultiLineCommentEnd,

  #[regex("[ \\t\\n\\r\\f\\v]+")]
  #[error]
  DontCare
}

pub struct CLexer<'a> {
  _syntax: Option<json::JsonValue>,
  _lex: Option<Vec<Vec<(CToken, std::ops::Range<usize>)>>>,
  _raw: Option<&'a Vec<Row>>
}

impl<'a> Lexer<'a> for CLexer<'a> {

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
      let mut multiline_flag = false;
      for row in rows {
        let mut row_lex = Vec::new();
        let lexed = CToken::lexer(row.content()).spanned();
        for token_range in lexed {
          match token_range.0 {
            CToken::MultiLineCommentStart => {
              multiline_flag = true;
              row_lex.push(token_range)
            },
            CToken::MultiLineCommentEnd => {
              multiline_flag = false;
              row_lex.push(token_range)
            }
            CToken::Function(name) => {
              if multiline_flag {
                row_lex.push((CToken::Comment, token_range.1));
                continue
              }
              row_lex.push((CToken::Function(name), std::ops::Range {
                start: token_range.1.start,
                end: token_range.1.end - 1
              }));
              row_lex.push((CToken::DontCare, std::ops::Range {
                start: token_range.1.end - 1,
                end: token_range.1.end
              }))
            },
            _ => {
              if multiline_flag {
                row_lex.push((CToken::Comment, token_range.1));
                continue
              }
              row_lex.push(token_range)
            }
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

fn match_color(token: &CToken, syntax_rules: &JsonValue) -> Option<Color> {
  let colors = &syntax_rules["colors"];
  match token {
    CToken::Keyword => get_color(colors["keyword"].as_str().unwrap()),
    CToken::Type => get_color(colors["type"].as_str().unwrap()),
    CToken::Char => get_color(colors["char"].as_str().unwrap()),
    CToken::String => get_color(colors["string"].as_str().unwrap()),
    CToken::Comment |
    CToken::MultiLineCommentStart |
    CToken::MultiLineCommentEnd => get_color(colors["comment"].as_str().unwrap()),
    CToken::Number => get_color(colors["number"].as_str().unwrap()),
    CToken::Function(_)  => get_color(colors["function"].as_str().unwrap()),
    _ => None
  }
}

fn get_attribute(token: &CToken, syntax_rules: &JsonValue) -> Attribute {
  let token_type = match token {
    CToken::Keyword => "keyword",
    CToken::Type => "type",
    CToken::Char => "char",
    CToken::String => "string",
    CToken::Comment |
    CToken::MultiLineCommentStart |
    CToken::MultiLineCommentEnd => "comment",
    CToken::Number => "number",
    CToken::Function(_)  => "function",
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

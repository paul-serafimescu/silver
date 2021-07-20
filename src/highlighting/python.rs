use crate::highlighting::{
  Lexer, Parsed, Row, Color,
  get_color, Attribute, Logos, LogosLexer,
  JsonValue
};

fn trim_function(token: &mut LogosLexer<PythonToken>) -> String {
  let mut string = token.slice().to_string();
  string.pop();
  string
}

#[derive(Debug, Logos, PartialEq)]
enum PythonToken {
  #[regex("\"([^\"]*)\"", priority = 100)]
  #[regex(r"'([^']*)'", priority = 99)]
  String,

  #[regex(r" -?[0-9]+(\.[0-9]+)?")]
  Number,

  #[regex(r"([a-zA-Z]+_?)*!?\(", priority = 98, callback = trim_function)]
  Function(String),

  #[token("import")]
  #[token("from")]
  #[token("if ")]
  #[token("else")]
  #[token("elif ")]
  #[token("while ")]
  #[token("for ")]
  #[token("class ")]
  #[token("break")]
  #[token("True")]
  #[token("False")]
  #[token("continue")]
  #[token("return")]
  #[token("pass")]
  #[token("try")]
  #[token("except")]
  #[token("finally")]
  #[token("as ")]
  #[token("def ")]
  #[token("raise ")]
  Keyword,

  #[token("dict")]
  #[token("list")]
  #[token("set")]
  #[token("int")]
  #[token("str")]
  #[token("float")]
  #[token("None")]
  #[token("bool")]
  #[token("bytes")]
  Type,

  #[regex(r"#.+", priority = 100)]
  Comment,

  #[token("\"\"\"")]
  MultiLineComment,

  #[regex(r"(([A-Z]+)_*)+")]
  #[regex(r"__[a-zA-Z]+__")]
  Constant,

  #[regex("[ \\t\\n\\r\\f\\v]+")]
  #[error]
  DontCare
}

pub struct PythonLexer<'a> {
  _syntax: Option<json::JsonValue>,
  _lex: Option<Vec<Vec<(PythonToken, std::ops::Range<usize>)>>>,
  _raw: Option<&'a Vec<Row>>
}

impl<'a> Lexer<'a> for PythonLexer<'a> {

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
        let lexed = PythonToken::lexer(row.content()).spanned();
        for token_range in lexed {
          match token_range.0 {
            PythonToken::MultiLineComment => {
              multiline_flag = !multiline_flag;
              row_lex.push(token_range)
            },
            PythonToken::Function(name) => {
              if multiline_flag {
                row_lex.push((PythonToken::Comment, token_range.1));
                continue
              }
              row_lex.push((PythonToken::Function(name), std::ops::Range {
                start: token_range.1.start,
                end: token_range.1.end - 1
              }));
              row_lex.push((PythonToken::DontCare, std::ops::Range {
                start: token_range.1.end - 1,
                end: token_range.1.end
              }))
            },
            _ => {
              if multiline_flag {
                row_lex.push((PythonToken::Comment, token_range.1));
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

fn match_color(token: &PythonToken, syntax_rules: &JsonValue) -> Option<Color> {
  let colors = &syntax_rules["colors"];
  match token {
    PythonToken::Keyword => get_color(colors["keyword"].as_str().unwrap()),
    PythonToken::Type => get_color(colors["type"].as_str().unwrap()),
    PythonToken::String => get_color(colors["string"].as_str().unwrap()),
    PythonToken::Comment |
    PythonToken::MultiLineComment => get_color(colors["comment"].as_str().unwrap()),
    PythonToken::Number => get_color(colors["number"].as_str().unwrap()),
    PythonToken::Function(_)  => get_color(colors["function"].as_str().unwrap()),
    PythonToken::Constant => get_color(colors["constant"].as_str().unwrap()),
    _ => None
  }
}

fn get_attribute(token: &PythonToken, syntax_rules: &JsonValue) -> Attribute {
  let token_type = match token {
    PythonToken::Keyword => "keyword",
    PythonToken::Type => "type",
    PythonToken::String => "string",
    PythonToken::Comment |
    PythonToken::MultiLineComment => "comment",
    PythonToken::Number => "number",
    PythonToken::Function(_)  => "function",
    PythonToken::Constant => "constant",
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

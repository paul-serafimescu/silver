use std::io::Write;
use std::env;
use std::fs::{read_to_string, OpenOptions};
use unicode_segmentation::UnicodeSegmentation;
use regex::Regex;
use crate::highlighting::*;

// newline position descriptor
pub enum NLPositionDescriptor {
  Beginning,
  Middle(usize),
  End
}

// insert position descriptor
pub enum IPositionDescriptor {
  Middle(usize, char),
  End(char)
}

// delete position descriptor
pub enum DPositionDescriptor {
  Beginning(usize),
  Middle(usize, usize),
  End(usize)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Row {
  content: String,
  len: usize,
  search_results: Vec<(usize, usize)>
}

impl AsRef<Row> for Row {
  fn as_ref(&self) -> &Self {
    &self
  }
}

impl Row {
  pub fn content(&self) -> &str {
    &self.content
  }

  pub fn len(&self) -> usize {
    self.len
  }

  #[allow(dead_code)]
  pub fn results(&self) -> &Vec<(usize, usize)> {
    &self.search_results
  }

  pub fn search_for(&mut self, expr: &String) -> usize {
    let mut counter = 0;
    let reg_expr = if let Ok(reg_expr) = Regex::new(expr) {
      reg_expr
    } else {
      return counter
    };
    self.search_results = reg_expr.find_iter(&self.content)
      .filter_map(|regex_match| Some((regex_match.start(), regex_match.end())))
      .collect();
    counter += self.search_results.len();
    counter
  }

  pub fn insert(&mut self, descrip: IPositionDescriptor) {
    match descrip {
      IPositionDescriptor::Middle(before, character) => {
        let mut new = String::new();
        for (idx, ch) in self.content.graphemes(true).enumerate() {
          if idx == before {
            new.push(character);
            new.push_str(ch);
            new.push_str(&self.content[(idx + 1)..self.content.len()]);
            break
          }
          new.push_str(ch)
        }
        self.content = new;
        self.len = self.content.graphemes(true).count()
      },
      IPositionDescriptor::End(character) => {
        self.content.push(character);
        self.len += 1
      }
    }
  }

  pub fn add_new_line(&mut self, at: NLPositionDescriptor) -> Row {
    let mut old = String::new();
    let mut new = String::new();
    match at {
      NLPositionDescriptor::Beginning => {
        let len = self.content.graphemes(true).count();
        let content = self.content.clone();
        let search_results = Vec::new();
        self.content = old;
        Row {
          content,
          len,
          search_results
        }
      },
      NLPositionDescriptor::Middle(at) => {
        for (idx, ch) in self.content.graphemes(true).enumerate() {
          old.push_str(ch);
          if idx == at {
            new.push_str(&self.content[(idx + 1)..self.content.len()]);
            break
          }
        }
        self.content = old;
        self.len = self.content.graphemes(true).count();
        let len = new.graphemes(true).count();
        let search_results = Vec::new();
        Row {
          content: new,
          len,
          search_results
        }
      },
      NLPositionDescriptor::End => {
        let len = new.graphemes(true).count();
        let search_results = Vec::new();
        Row {
          content: new,
          len,
          search_results
        }
      }
    }
  }

  pub fn delete(&mut self, at: usize) {
    let mut new = String::new();
    for (idx, ch) in self.content.graphemes(true).enumerate() {
      if idx == at {
        new.push_str(&self.content[(idx + 1)..self.content.len()]);
        break
      }
      new.push_str(ch)
    }
    self.len -= 1;
    self.content = new;
  }

  pub fn append(&mut self, other: &Self) {
    self.len += other.content.graphemes(true).count();
    self.content.push_str(&other.content)
  }

  pub fn pop(&mut self) -> Option<char> {
    self.len -= 1;
    self.content.pop()
  }
}

impl From<&str> for Row {
  fn from(string: &str) -> Self {
    Row {
      content: String::from(string),
      len: string.graphemes(true).count(),
      search_results: Vec::new()
    }
  }
}

#[derive(Debug)]
pub struct Document {
  pub file_name: String,
  pub rows: Vec<Row>,
  pub syntax_file: Option<JsonValue>,
  pub highlighted_rows: Option<Vec<Vec<Parsed>>>
}

impl Document {
  pub fn open(file_name: &str) -> Result<Self, std::io::Error> {
    let raw_content = read_to_string(file_name)?;
    let file_name = String::from(file_name);
    let mut rows = Vec::new();
    for line in raw_content.lines() {
      rows.push(Row::from(line));
    }
    let path = env::current_dir().unwrap().join("syntax/rust.json");
    let syntax_file = if let Ok(file_contents) = read_to_string(path) {
      if let Ok(result) = json::parse(&file_contents) {
        if !result["highlight"].as_bool().unwrap() {
          None
        } else {
          Some(result)
        }
      } else {
        None
      }
    } else {
      None
    };
    let highlighted_rows = highlight(&file_name, &rows, &syntax_file);
    Ok(Self {
      file_name,
      rows,
      syntax_file,
      highlighted_rows
    })
  }

  pub fn new(file_name: &str) -> Self {
    let file_name = String::from(file_name);
    let mut rows = Vec::new();
    rows.push(Row::from(""));
    let path = env::current_dir().unwrap().join("syntax/rust.json");
    let syntax_file = if let Ok(file_contents) = read_to_string(path) {
      if let Ok(result) = json::parse(&file_contents) {
        if !result["highlight"].as_bool().unwrap() {
          None
        } else {
          Some(result)
        }
      } else {
        None
      }
    } else {
      None
    };
    let highlighted_rows = highlight(&file_name, &rows, &syntax_file);
    Self {
      file_name,
      rows,
      syntax_file,
      highlighted_rows
    }
  }

  pub fn name(&self) -> &str {
    &self.file_name
  }

  pub fn highlighted_rows(&self) -> &Option<Vec<Vec<Parsed>>> {
    &self.highlighted_rows
  }

  pub fn set_name(&mut self, name: &str) {
    self.file_name = String::from(name)
  }

  pub fn get_row(&self, index: usize) -> Option<&Row> {
    self.rows.get(index)
  }

  pub fn clear_row(&mut self, index: usize) {
    let row = self.rows.get_mut(index).unwrap();
    row.content.clear();
    row.len = 0
  }

  pub fn get_row_mut(&mut self, index: usize) -> Result<&mut Row, ()> {
    if let Some(row) = self.rows.get_mut(index) {
      Ok(row)
    } else {
      Err(())
    }
  }

  pub fn len(&self) -> usize {
    self.rows.len()
  }

  pub fn to_str(&self) -> String {
    let mut stringified = String::new();
    for row in &self.rows {
      stringified.push_str(row.content());
      stringified.push('\n')
    }
    stringified
  }

  pub fn save(&self) -> Result<usize, std::io::Error> {
    let mut file = OpenOptions::new().write(true).truncate(true).create(true).open(self.file_name.as_str())?;
    file.write(self.to_str().as_bytes())
  }

  pub fn insert_row(&mut self, row_no: usize, row: Row) {
    self.rows.insert(row_no, row)
  }

  pub fn handle_delete(&mut self, descrip: DPositionDescriptor) -> Option<usize> {
    match descrip {
      DPositionDescriptor::Middle(row_no, at) => {
        self.get_row_mut(row_no).unwrap().delete(at);
        None
      },
      DPositionDescriptor::Beginning(row_no) => {
        if row_no == 0 {
          return None
        }
        let row = self.rows.remove(row_no);
        let prev_row = self.rows.get_mut(row_no - 1).unwrap();
        let prev_len = prev_row.len();
        prev_row.append(&row);
        Some(prev_len)
      },
      DPositionDescriptor::End(row_no) => {
        self.get_row_mut(row_no).unwrap().pop();
        None
      }
    }
  }

  pub fn highlight(&mut self) {
    self.highlighted_rows = highlight(&self.file_name, &self.rows, &self.syntax_file);
  }

  pub fn replace(&mut self, index: usize, new_row: Row) {
    self.rows[index] = new_row;
  }

  pub fn search_for(&mut self, expr: &String) -> usize {
    let mut counter = 0;
    for row in &mut self.rows {
      counter += row.search_for(expr)
    }
    counter
  }
}

// "static" helper functions

pub fn highlight(file_name: &str, rows: &Vec<Row>, syntax_file: &Option<JsonValue>) -> Option<Vec<Vec<Parsed>>> {
  if let Some(extension) = file_name.split('.').collect::<Vec<&str>>().last() {
    match *extension {
      "rs" => RustLexer::lex(&rows, syntax_file.as_ref()).parse(),
      "py" => PythonLexer::lex(&rows, syntax_file.as_ref()).parse(),
      _ => None
    }
  } else { None }
}

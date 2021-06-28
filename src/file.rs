#![allow(dead_code)]

use std::io::Write;
use std::fs::{read_to_string, OpenOptions};
use unicode_segmentation::UnicodeSegmentation;

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

#[derive(Debug)]
pub struct Row {
  content: String,
  len: usize,
}

impl Row {
  pub fn content(&self) -> &str {
    &self.content
  }

  pub fn len(&self) -> usize {
    self.len
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
        self.content = old;
        Row {
          content,
          len
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
        Row {
          content: new,
          len
        }
      },
      NLPositionDescriptor::End => {
        let len = new.graphemes(true).count();
        Row {
          content: new,
          len
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
      len: string.graphemes(true).count()
    }
  }
}

#[derive(Debug)]
pub struct Document {
  pub file_name: String,
  pub rows: Vec<Row>
}

impl Document {
  pub fn open(file_name: &str) -> Result<Self, std::io::Error> {
    let raw_content = read_to_string(file_name)?;
    let file_name = String::from(file_name);
    let mut rows = Vec::new();
    for line in raw_content.lines() {
      rows.push(Row::from(line));
    }
    Ok(Self {
      file_name,
      rows
    })
  }

  pub fn new(file_name: &str) -> Self {
    let file_name = String::from(file_name);
    let mut rows = Vec::new();
    rows.push(Row::from(""));
    Self {
      file_name,
      rows
    }
  }

  pub fn name(&self) -> &str {
    &self.file_name
  }

  pub fn set_name(&mut self, name: &str) {
    self.file_name = String::from(name)
  }

  pub fn get_row(&self, index: usize) -> Option<&Row> {
    self.rows.get(index)
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
}

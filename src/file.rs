#![allow(dead_code)]

use std::io::Write;
use std::fs::{read_to_string, OpenOptions};
use unicode_segmentation::UnicodeSegmentation;

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

  pub fn insert(&mut self, before: usize, character: char) {
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
    self.content = new;
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

  pub fn get_row(&self, index: usize) -> Result<&Row, ()> {
    if let Some(row) = self.rows.get(index) {
      Ok(row)
    } else {
      Err(())
    }
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

  fn to_str(&self) -> String {
    let mut stringified = String::new();
    for row in &self.rows {
      stringified.push_str(row.content());
      stringified.push('\n')
    }
    stringified
  }

  pub fn save(&self) -> Result<usize, std::io::Error> {
    let mut file = OpenOptions::new().write(true).create(true).open(self.file_name.as_str())?;
    file.write(self.to_str().as_bytes())
  }
}

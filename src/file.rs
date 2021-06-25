#![allow(dead_code)]

use std::fs::{read_to_string};
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

  pub fn len(&self) -> usize {
    self.rows.len()
  }
}

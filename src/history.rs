#![allow(dead_code)]

use std::ops::Range;
use std::collections::VecDeque;
use crate::file::{
  Row
};

#[derive(Debug, PartialEq)]
pub struct HistoryNode {
  pub altered_rows: Range<usize>,
  pub rows: Vec<Row>
}

impl HistoryNode {
  pub fn create(rows: &[Row], altered_rows: Range<usize>) -> Self {
    let mut r = Vec::new();
    for row in rows {
      r.push(row.clone())
    }
    Self {
      altered_rows,
      rows: r
    }
  }

  pub fn extract(self) -> (Range<usize>, Vec<Row>) {
    (self.altered_rows, self.rows)
  }
}

#[derive(Debug, PartialEq)]
pub struct History {
  history: VecDeque<HistoryNode>,
  maximum_size: usize
}

impl<'a> History {
  pub fn new() -> Self {
    Self {
      history: VecDeque::new(),
      maximum_size: 50
    }
  }

  pub fn with_capacity(maximum_size: usize) -> Self {
    Self {
      history: VecDeque::new(),
      maximum_size
    }
  }

  pub fn capacity(&self) -> usize {
    self.maximum_size
  }

  pub fn push(&mut self, node: HistoryNode) {
    if self.history.len() == self.maximum_size {
      self.history.truncate(self.maximum_size)
    }
    self.history.push_front(node)
  }

  pub fn pop(&mut self) -> Option<HistoryNode> {
    self.history.pop_front()
  }
}

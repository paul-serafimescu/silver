#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_macros)]

use std::io::{stdout, Stdout, Write};
use std::fs::{File, read_to_string};
use crossterm::{
  terminal::{
    enable_raw_mode,
    disable_raw_mode,
    size, Clear, ClearType,
    EnterAlternateScreen,
    LeaveAlternateScreen,
  },
  event::{
    KeyCode, Event,
    read, KeyEvent,
    KeyModifiers,
  },
  cursor::{
    MoveTo
  }, execute,
};
use super::file::{Document, Row};

const OFFSET: u8 = 1;
const NONE: KeyModifiers = KeyModifiers::empty();

macro_rules! char_key {
  ($key: expr) => {
    Event::Key(KeyEvent {
      code: KeyCode::Char($key),
      modifiers: NONE
    })
  };
}

macro_rules! special_key {
  ($en_t: pat) => {
    Event::Key(KeyEvent {
      code: $en_t,
      modifiers: NONE
    })
  };
}

#[derive(Debug)]
pub struct Terminal {
  height: u16,
  width: u16,
}

impl Terminal {
  pub fn new() -> Result<Self, std::io::Error> {
    let dimensions = size()?;
    enable_raw_mode()?;
    execute!(
      stdout(),
      EnterAlternateScreen
    )?;
    Ok(Self {
      height: dimensions.1,
      width: dimensions.0,
    })
  }

  pub fn size(&self) -> (u16, u16) {
    (self.width, self.height)
  }

  pub fn clear(&self) -> Result<(), std::io::Error> {
    Ok(execute!(
      stdout(),
      Clear(ClearType::All)
    )?)
  }
}

impl Drop for Terminal {
  fn drop(&mut self) {
    let _ = disable_raw_mode();
    let _ = execute!(
      stdout(),
      LeaveAlternateScreen
    );
  }
}

#[derive(Debug)]
pub enum EditorMode {
  Normal,
  Command,
  Insert
}

impl EditorMode {
  pub fn to_string(&self) -> &str {
    match self {
      EditorMode::Normal => "NORMAL",
      EditorMode::Command => "COMMAND",
      EditorMode::Insert => "INSERT"
    }
  }
}

#[derive(Debug)]
pub struct Editor {
  pub terminal: Terminal,
  pub file: Option<Document>,
  pub mode: EditorMode,
  pub status_bar: String,
  _quit: bool
}

impl Editor {
  pub fn default() -> Result<Self, std::io::Error> {
    Ok(Editor {
      terminal: Terminal::new()?,
      file: None,
      _quit: false,
      mode: EditorMode::Normal,
      status_bar: format!("{}{}",
        (0..(size()?.0 as usize - 6))
          .map(|_| " ")
          .collect::<String>(),
        "NORMAL")
    })
  }

  pub fn new(file_name: &str) -> Result<Self, std::io::Error> {
    Ok(Editor {
      terminal: Terminal::new()?,
      file: Some(Document::open(file_name)?),
      _quit: false,
      mode: EditorMode::Normal,
      status_bar: format!("{}{}",
        (0..(size()?.0 as usize - 6))
          .map(|_| " ")
          .collect::<String>(),
        "NORMAL")
    })
  }

  pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(contents) = &self.file {
      let num_rows = contents.rows.len();
      let buffer = num_rows.to_string().chars().count();
      for terminal_row_no in 0..(self.terminal.size().1 - 1) as usize {
        if terminal_row_no < num_rows {
          let used = buffer - (terminal_row_no + 1).to_string().chars().count();
          self.write_row(terminal_row_no + 1, used, contents.rows.get(terminal_row_no).unwrap());
        } else {
          self.write_empty_line();
        }
      }
    } else {

    }
    loop {
      if self._quit {
        return Ok(())
      }
      self.render_status_bar();
      match &self.mode {
        EditorMode::Normal => self.handle_normal(),
        EditorMode::Command => self.handle_command(),
        EditorMode::Insert => self.handle_insert()
      }
    }
  }

  fn quit(&mut self) {
    self._quit = true
  }

  fn write_row(&self, row_no: usize, offset: usize, row: &Row) {
    print!("{:indent$}{} {}\r\n", "", row_no, row.content(), indent=offset)
  }

  fn write_empty_line(&self) {
    print!("~\r\n")
  }

  fn handle_command(&mut self) {
    match read().unwrap() {
      char_key!('q') => self.quit(),
      special_key!(KeyCode::Esc) => {
        self.mode = EditorMode::Normal
      },
      _ => ()
    }
  }

  fn handle_insert(&mut self) {
    match read().unwrap() {
      special_key!(KeyCode::Esc) => {
        self.mode = EditorMode::Normal
      },
      _ => () // TODO: all the insert operations, refreshing the buffer
    }
  }

  fn handle_normal(&mut self) {
    match read().unwrap() {
      char_key!('i') => self.mode = EditorMode::Insert,
      char_key!(':') => {
        self.mode = EditorMode::Command;
        self.status_bar = format!(":{}", &self.status_bar[1..]);
      },
      _ => ()
    }
  }

  fn render_status_bar(&self) {
    execute!(
      stdout(),
      MoveTo(0, self.terminal.size().0),
      Clear(ClearType::CurrentLine)
    ).unwrap();
    print!("{}", self.status_bar);
    stdout().flush().unwrap();
  }
}

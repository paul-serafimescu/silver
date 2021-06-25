use std::io::{stdout, Write};
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
    MoveTo, Hide, Show,
    SavePosition, RestorePosition,
    position
  }, execute,
};
use super::file::{Document, Row};

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

  #[allow(dead_code)]
  pub fn clear(&self) -> Result<(), std::io::Error> {
    Ok(execute!(
      stdout(),
      Clear(ClearType::All)
    )?)
  }
}

#[derive(Debug)]
pub enum EditorMode {
  Normal,
  Command,
  Insert
}

impl EditorMode {
  pub fn to_string(&self) -> String {
    match self {
      EditorMode::Normal => String::from("NORMAL"),
      EditorMode::Command => String::from("COMMAND"),
      EditorMode::Insert => String::from("INSERT")
    }
  }
}

#[derive(Debug)]
pub enum Direction {
  Down,
  Up,
  Left,
  Right
}

#[derive(Debug)]
pub struct StatusBar {
  pub terminal_size: (u16, u16),
  pub cmd: String,
  pub cmd_chars: usize,
  pub mode: EditorMode,
}

impl StatusBar {
  pub fn default() -> Self {
    let dimensions = size().unwrap();
    Self {
      terminal_size: dimensions,
      cmd: String::new(),
      cmd_chars: 0,
      mode: EditorMode::Normal
    }
  }

  pub fn add_command(&mut self, command: char) {
    self.cmd_chars += 1;
    self.cmd.push(command);
    self.render();
  }

  pub fn set_mode(&mut self, mode: EditorMode) {
    self.mode = mode
  }

  pub fn render(&self) {
    let mode_str = self.mode.to_string();
    let content = format!("{}{}{}",
      self.cmd,
      (self.cmd_chars..(self.terminal_size.0 as usize - mode_str.len()))
        .map(|_| " ")
        .collect::<String>(),
      mode_str);
    print!("{}\r", content);
  }
}

#[derive(Debug)]
pub struct Editor {
  pub terminal: Terminal,
  pub file: Option<Document>,
  pub mode: EditorMode,
  pub status_bar: StatusBar,
  view_frame: (usize, usize),
  _quit: bool,
  position: (u16, u16),
  buffer: u16
}

impl Editor {
  pub fn default() -> Result<Self, std::io::Error> {
    let terminal = Terminal::new()?;
    let terminal_rows = terminal.size().1;
    Ok(Editor {
      terminal,
      file: None,
      _quit: false,
      mode: EditorMode::Normal,
      status_bar: StatusBar::default(),
      view_frame: (0, terminal_rows as usize),
      position: (0, 0),
      buffer: 0
    })
  }

  pub fn new(file_name: &str) -> Result<Self, std::io::Error> {
    let terminal = Terminal::new()?;
    let terminal_rows = terminal.size().1;
    Ok(Editor {
      terminal,
      file: Some(Document::open(file_name)?),
      _quit: false,
      mode: EditorMode::Normal,
      status_bar: StatusBar::default(),
      view_frame: (0, terminal_rows as usize),
      position: (0, 0),
      buffer: 0
    })
  }

  pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    loop {
      if self._quit {
        return Ok(())
      }
      self.render();
      match &self.mode {
        EditorMode::Normal => self.handle_normal(),
        EditorMode::Command => self.handle_command(),
        EditorMode::Insert => self.handle_insert()
      }
      // apparently i'm handling all the main stuff too fast for my terminal
      // the source of the flickering needs to be resolved ASAP
      std::thread::sleep(std::time::Duration::from_millis(1));
    }
  }

  fn write_row(&self, row_no: usize, offset: usize, row: &Row) {
    print!("{:indent$}{} {}\r\n", "", row_no, row.content(), indent=offset)
  }

  fn write_empty_line(&self) {
    self.clear_row();
    print!("~\r\n")
  }

  fn handle_command(&mut self) {
    match read().unwrap() {
      char_key!('q') => {
        self.status_bar.add_command('q');
      },
      special_key!(KeyCode::Enter) => {
        self.evaluate_expr();
        self.mode = EditorMode::Normal;
        self.status_bar.set_mode(EditorMode::Normal)
      },
      special_key!(KeyCode::Esc) => {
        self.mode = EditorMode::Normal;
        self.status_bar.set_mode(EditorMode::Normal)
      },
      _ => ()
    }
  }

  fn handle_insert(&mut self) {
    match read().unwrap() {
      special_key!(KeyCode::Esc) => {
        self.mode = EditorMode::Normal;
        self.status_bar.set_mode(EditorMode::Normal)
      },
      _ => () // TODO: all the insert operations, refreshing the buffer
    }
  }

  fn handle_normal(&mut self) {
    match read().unwrap() {
      char_key!('i') => {
        self.mode = EditorMode::Insert;
        self.status_bar.set_mode(EditorMode::Insert)
      },
      char_key!(':') => {
        self.mode = EditorMode::Command;
        self.status_bar.set_mode(EditorMode::Command);
        self.status_bar.add_command(':');
      },
      special_key!(KeyCode::Down) => {
        self.scroll(Direction::Down);
      },
      special_key!(KeyCode::Up) => {
        self.scroll(Direction::Up);
      },
      special_key!(KeyCode::Left) => {
        self.scroll(Direction::Left)
      },
      special_key!(KeyCode::Right) => {
        self.scroll(Direction::Right)
      }
      _ => ()
    }
  }

  fn evaluate_expr(&mut self) {
    for cmd in self.status_bar.cmd.chars() {
      match cmd {
        'q' => self._quit = true,
        _ => ()
      }
    }
  }

  fn scroll(&mut self, direction: Direction) {
    let _ = execute!(
      stdout(),
      Hide
    );
    match direction {
      Direction::Down => {
        if self.view_frame.1 - 2 < self.file.as_ref().unwrap().rows.len() as usize
        || self.position.0 + 2 != self.terminal.height {
          if self.position.0 == self.terminal.size().1 - 2 {
            self.view_frame = (self.view_frame.0 + 1, self.view_frame.1 + 1);
          } else {
            self.position.0 += 1;
            let _ = execute!(stdout(), MoveTo(self.position.1, self.position.0));
          }
        }
      },
      Direction::Up => {
        if self.view_frame.0 != 0 || self.position.0 != 0 {
          if self.position.0 == 0 {
            self.view_frame = (self.view_frame.0 - 1, self.view_frame.1 - 1);
          } else {
            self.position.0 -= 1;
            let _ = execute!(stdout(), MoveTo(self.position.1, self.position.0));
          }
        }
      },
      Direction::Left => {
        if self.position.1 > self.buffer + 1 {
          self.position.1 -= 1;
          let _ = execute!(stdout(), MoveTo(self.position.1, self.position.0));
        }
      },
      Direction::Right => {
        if self.position.1 != self.terminal.width {
          if let Ok(row) = self.file.as_ref().unwrap().get_row(self.view_frame.0 + self.position.0 as usize) {
            if row.len() + 3 == self.position.1 as usize {
              self.scroll(Direction::Down);
              self.position.1 = self.buffer + 1;
            } else {
              self.position.1 += 1;
            }
          }
          let _ = execute!(stdout(), MoveTo(self.position.1, self.position.0));
        }
      }
    }
  }

  fn clear_row(&self) {
    let _ = execute!(
      stdout(),
      Clear(ClearType::CurrentLine)
    );
  }

  fn render(&mut self) {
    let _ = execute!(
      stdout(),
      Hide,
      SavePosition,
      MoveTo(0, 0),
    );
    if let Some(contents) = &self.file {
      let num_rows = contents.rows.len();
      let buffer = num_rows.to_string().chars().count();
      self.buffer = buffer as u16;
      for terminal_row_no in self.view_frame.0..(self.view_frame.1 - 1) {
        if terminal_row_no < num_rows {
          self.clear_row();
          let used = buffer - (terminal_row_no + 1).to_string().chars().count();
          self.write_row(terminal_row_no + 1, used, contents.rows.get(terminal_row_no).unwrap());
        } else {
          self.write_empty_line();
        }
      }
    } else {

    }
    self.status_bar.render();
    match self.mode {
      EditorMode::Command => (),
      _ => {
        let _ = execute!(stdout(), RestorePosition, Show);
        let (column, row) = position().unwrap();
        if column == 0 {
          self.position.1 = self.buffer + 1;
          let _ = execute!(
            stdout(),
            MoveTo(self.position.1, row)
          );
        }
      }
    }
    let _ = stdout().flush();
  }
}

impl Drop for Editor {
  fn drop(&mut self) {
    let _ = disable_raw_mode();
    let _ = execute!(
      stdout(),
      LeaveAlternateScreen
    );
    println!("row {} column {}", self.position.0, self.position.1);
  }
}

use std::io::{stdout, Write};
use std::panic;
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
  style::{
    SetForegroundColor, SetBackgroundColor,
    ResetColor,
    Color,
    Print, SetAttribute
  },
  cursor::{
    MoveTo, Hide, Show,
    SavePosition, RestorePosition,
    position, SetCursorShape, CursorShape,
    EnableBlinking
  }, execute
};
use crate::file::{
  Document, Row,
  NLPositionDescriptor, DPositionDescriptor,
  IPositionDescriptor
};
use crate::history::*;

const NONE: KeyModifiers = KeyModifiers::empty();
const UPPER: KeyModifiers = KeyModifiers::SHIFT;
#[allow(dead_code)]
const VERSION: &str = env!("CARGO_PKG_VERSION");

macro_rules! char_key {
  ($key: pat) => {
    Event::Key(KeyEvent {
      code: KeyCode::Char($key),
      modifiers: NONE
    })
  };
}

macro_rules! char_upper_key {
  ($key: pat) => {
    Event::Key(KeyEvent {
      code: KeyCode::Char($key),
      modifiers: UPPER
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

  pub fn set_dimensions(&mut self) {
    let (width, height) = size().unwrap();
    self.width = width;
    self.height = height
  }

  pub fn clear(&self) -> Result<(), std::io::Error> {
    Ok(execute!(
      stdout(),
      Clear(ClearType::All)
    )?)
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorMode {
  Normal,
  Command,
  Insert,
  Search
}

impl EditorMode {
  pub fn to_string(&self) -> String {
    String::from(match self {
      EditorMode::Normal => "VIEW",
      EditorMode::Command => "COMMAND",
      EditorMode::Insert => "INSERT",
      EditorMode::Search => "SEARCH"
    })
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
      mode: EditorMode::Normal,
    }
  }

  pub fn add_command(&mut self, command: char) {
    self.cmd_chars += 1;
    self.cmd.push(command);
  }

  pub fn remove_command(&mut self) {
    self.cmd_chars -= 1;
    self.cmd.pop();
  }

  pub fn set_mode(&mut self, mode: &EditorMode) {
    self.mode = *mode
  }

  pub fn render(&mut self, current: usize, total: usize) {
    self.terminal_size = size().unwrap();
    let mode_str = self.mode.to_string();
    let line_chars = current.to_string().chars().count() + total.to_string().chars().count();
    let mut stdout = stdout();
    let content = format!("{}{}{}",
      self.cmd,
      (self.cmd_chars..(self.terminal_size.0 as usize - mode_str.len() - line_chars - 4))
        .map(|_| " ")
        .collect::<String>(),
      mode_str);
    let _ = execute!(
      stdout,
      Print(format!("{} | ", content)),
      SetBackgroundColor(Color::White),
      SetForegroundColor(Color::Black),
      Print(format!("{}/{}\r", current, total)),
      ResetColor
    );
  }
}

#[derive(Debug)]
pub struct Editor {
  pub terminal: Terminal,
  pub file: Document,
  pub mode: EditorMode,
  pub status_bar: StatusBar,
  pub history: History,
  pub search_results: Option<std::vec::IntoIter<(usize, usize)>>,
  _old_position: (u16, u16),
  altered: bool,
  view_frame: (usize, usize),
  _quit: bool,
  _search_current: usize,
  _search_total: usize,
  position: (u16, u16),
  buffer: u16
}

impl Editor {
  pub fn new(file_name: Option<&String>) -> Result<Self, std::io::Error> {
    panic::set_hook(Box::new(|_| {
      let _ = execute!(
        stdout(),
        // LeaveAlternateScreen,
        ResetColor,
        SetCursorShape(CursorShape::Block),
        EnableBlinking
      );
      let _ = disable_raw_mode();
    }));
    let terminal = Terminal::new()?;
    let terminal_rows = terminal.size().1;
    Ok(Editor {
      terminal,
      altered: false,
      file: if let Some(file_name) = file_name {
        if let Ok(file) = Document::open(file_name) {
          file
        } else {
          Document::new(file_name)
        }
      } else { Document::new("") },
      _quit: false,
      mode: EditorMode::Normal,
      status_bar: StatusBar::default(),
      view_frame: (0, terminal_rows as usize),
      position: (0, 0),
      buffer: 0,
      history: History::new(),
      search_results: None,
      _search_current: 0,
      _search_total: 0,
      _old_position: position()?,
    })
  }

  pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    // anything that needs to be done on boot
    execute!(
      stdout(),
      SetCursorShape(CursorShape::Block),
      EnableBlinking,
      MoveTo(0, 0)
    )?;
    loop {
      if self._quit {
        return Ok(())
      }
      let old_view_frame = self.view_frame;
      let old_num_rows = self.view_frame.1 - self.view_frame.0;
      self.terminal.set_dimensions();
      self.view_frame = (old_view_frame.0, old_view_frame.1 + self.terminal.height as usize - old_num_rows);
      self.render();
      match &self.mode {
        EditorMode::Normal => self.handle_normal(),
        EditorMode::Command => self.handle_command(),
        EditorMode::Insert => self.handle_insert(),
        EditorMode::Search => self.handle_search()
      }
      std::thread::sleep(std::time::Duration::from_millis(1));
    }
  }

  fn write_row(&self, row_no: usize, offset: usize, row: &Row) {
    let mut printed_string = String::from(row.content());
    let mut current_written = 0;
    printed_string.truncate((self.terminal.width - self.buffer - 1) as usize);
    let mut stdout = stdout();
    execute!(
      stdout,
      Print(format!("{:indent$}{} ", "", row_no, indent=offset))
    ).unwrap();
    if let Some(highlighted_rows) = self.file.highlighted_rows() {
      for token in highlighted_rows.get(row_no - 1).unwrap() {
        current_written += token.get_original().chars().count(); // temporary
        if current_written > (self.terminal.width - self.buffer - 1) as usize { // temporary
          break
        }
        if let (Some(color), attribute) = token.get_color_and_attribute() {
          execute!(
            stdout,
            SetForegroundColor(*color),
            SetAttribute(*attribute),
            Print(format!("{}", token.get_original())),
            ResetColor
          ).unwrap();
        } else {
          execute!(
            stdout,
            Print(format!("{}", token.get_original()))
          ).unwrap();
        }
      }
      execute!(
        stdout,
        Print("\r\n")
      ).unwrap();
    } else {
      execute!(
        stdout,
        Print(format!("{}\r\n", printed_string))
      ).unwrap();
    }
  }

  fn write_empty_line(&self) {
    self.clear_row();
    execute!(
      stdout(),
      Print("~\r\n")
    ).unwrap();
  }

  fn set_buffer(&mut self) {
    self.buffer = self.file.len().to_string().chars().count() as u16;
  }

  fn move_to(&mut self, column: u16, row: u16) {
    execute!(
      stdout(),
      MoveTo(column, row)
    ).unwrap();
    self.position.0 = row;
    self.position.1 = column
  }

  fn handle_command(&mut self) {
    match read().unwrap() {
      char_key!(key) => {
        self.status_bar.add_command(key);
      },
      char_upper_key!(key) => {
        self.status_bar.add_command(key)
      },
      special_key!(KeyCode::Enter) => {
        self.move_to(self.position.1, self.position.0);
        if let Ok(_) = self.evaluate_expr() {
          self.set_mode(EditorMode::Normal)
        }
      },
      special_key!(KeyCode::Esc) => {
        self.set_mode(EditorMode::Normal);
        self.status_bar.cmd.clear();
        self.status_bar.cmd_chars = 0
      },
      special_key!(KeyCode::Backspace) => {
        if self.status_bar.cmd.len() > 1 {
          self.status_bar.remove_command()
        }
      }
      _ => ()
    }
  }

  fn handle_insert(&mut self) {
    match read().unwrap() {
      char_key!(key) | char_upper_key!(key) => self.insert(key),
      special_key!(KeyCode::Tab) => {
        self.insert(' ');
        self.insert(' ') // yeah i'm forcing you to use 2 space tabs
      },
      special_key!(KeyCode::Backspace) => self.delete(),
      special_key!(KeyCode::Enter) => self.insert_row(),
      special_key!(KeyCode::Esc) => self.set_mode(EditorMode::Normal),
      special_key!(KeyCode::Down) => self.scroll(Direction::Down),
      special_key!(KeyCode::Up) => self.scroll(Direction::Up),
      special_key!(KeyCode::Left) => self.scroll(Direction::Left),
      special_key!(KeyCode::Right) => self.scroll(Direction::Right),
      _ => () // TODO: all the insert operations, refreshing the buffer
    }
    self.altered = true;
  }

  fn handle_normal(&mut self) {
    match read().unwrap() {
      char_key!('i') => self.set_mode(EditorMode::Insert),
      char_key!(':') => {
        self.set_mode(EditorMode::Command);
        self.status_bar.add_command(':');
      },
      special_key!(KeyCode::Down) => {
        self.scroll(Direction::Down)
      },
      special_key!(KeyCode::Up) => {
        self.scroll(Direction::Up)
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

  fn handle_search(&mut self) {
    match read().unwrap() {
      special_key!(KeyCode::Esc) => self.set_mode(EditorMode::Normal),
      special_key!(KeyCode::Enter) => {
        if let Some(search_results) = &mut self.search_results {
          if let Some((row_idx, row_offset)) = search_results.next() {
            self._search_current += 1;
            self.goto_line(row_idx + 1);
            self.move_to(row_offset as u16 + self.buffer + 1, self.position.0)
          } else {
            self.set_mode(EditorMode::Normal)
          }
        } else {
          self.set_mode(EditorMode::Normal)
        }
      },
      char_key!('i') => self.set_mode(EditorMode::Insert),
      _ => ()
    }
  }

  fn evaluate_expr(&mut self) -> Result<(), ()> {
    let mut next_mode_not_normal = false;
    if self.status_bar.cmd.starts_with(":set") {
      let copied_cmd = self.status_bar.cmd
      .clone();
      let split_command = copied_cmd.split_whitespace().collect::<Vec<&str>>();
      for idx in 1..split_command.len() {
        match *split_command.get(idx).unwrap() {
          ":set" => (),
          "line" => {
            if let Some(line_no) = split_command.get(idx + 1) {
              if let Ok(line_no) = line_no.parse::<usize>() {
                self.goto_line(line_no);
              }
            }
            break
          }
          "filename" => {
            if let Some(file_name) = split_command.get(idx + 1) {
              self.file.set_name(*file_name)
            }
            break
          },
          _ => ()
        }
      }
    } else {
      let mut commands = self.status_bar.cmd.chars().rev().collect::<String>();
      while let Some(cmd) = commands.pop() {
        match cmd {
          'q' => if self.file.name() != "" || !self.altered { self._quit = true },
          'e' => self.move_to_line_end(),
          'a' => self.move_to_line_beginning(),
          'A' => {
            self.move_to_line_end();
            self.set_mode(EditorMode::Insert);
            next_mode_not_normal = true;
          },
          'w' => {
            for _ in 0..numeric_modifer(&mut commands) {
              self.move_to_next_word()
            }
          },
          'b' => {
            for _ in 0..numeric_modifer(&mut commands) {
              self.move_to_prev_word()
            }
          },
          'i' => {
            self.set_mode(EditorMode::Insert);
            next_mode_not_normal = true
          },
          'd' => {
            let num_lines = numeric_modifer(&mut commands);
            for _ in 0..num_lines {
              let row_no = self.position.0 as usize + self.view_frame.0;
              self.history.push(HistoryNode::create(&self.file.rows[row_no..(row_no + 1)], row_no..(row_no + 1)));
              self.file.clear_row(row_no);
              self.move_to_line_beginning();
              self.scroll(Direction::Down)
            }
            self.move_to_line_beginning()
          },
          'g' => self.move_to_beginning(),
          'G' => self.move_to_end(),
          'x' => {
            self._quit = true;
            self.altered = false;
          },
          'u' => {
            let reps = numeric_modifer(&mut commands);
            for _ in 0..reps {
              self.undo()
            }
          },
          '/' => {
            if let Some(arg) = word_modifier(&mut commands) {
              next_mode_not_normal = true;
              self.search(arg);
              if let Some(result_iter) = &mut self.search_results {
                if let Some((row_no, row_offset)) = result_iter.next() {
                  self.goto_line(row_no + 1);
                  self.move_to(row_offset as u16 + self.buffer + 1, self.position.0)
                } else {
                  self.set_mode(EditorMode::Normal);
                  next_mode_not_normal = false
                }
              } else {
                next_mode_not_normal = false
              }
            }
            break
          }
          _ => ()
        }
      }
    }
    self.status_bar.cmd.clear();
    self.status_bar.cmd_chars = 0;

    if next_mode_not_normal {
      Err(())
    } else {
      Ok(())
    }
  }

  fn set_mode(&mut self, mode: EditorMode) {
    self.status_bar.set_mode(&mode);
    self.set_cursor(&mode);
    self.mode = mode;
  }

  fn goto_line(&mut self, line_no: usize) {
    if line_no as usize <= self.file.len() {
      let difference = line_no as i64 - self.position.0 as i64 - self.view_frame.0 as i64;
      if difference <= 0 {
        for _ in 0..(difference.abs() + 1) {
          self.scroll(Direction::Up)
        }
      } else {
        for _ in 0..(difference - 1) {
          self.scroll(Direction::Down)
        }
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
        let file = &self.file;
        // ensure that we are within the bounds of the file,
        // add one null line to allow buffer to grow
        if self.view_frame.1 - 2 < file.rows.len() as usize
        || self.position.0 + 2 != self.terminal.height {
          // grab row below the current cursor row
          if let Some(row) = file.get_row(self.position.0 as usize + self.view_frame.0 + 1) {
            // 'next_column' ensures stickiness to the left
            let next_column = std::cmp::min(self.position.1, 1 + self.buffer + row.len() as u16);
            // we're at the bottom of the view frame, scroll one line down, move everything else
            if self.position.0 == self.terminal.size().1 - 2 {
              self.view_frame = (self.view_frame.0 + 1, self.view_frame.1 + 1);
              self.position = (self.position.0, next_column);
            } else { // stick left, move cursor left depending on the character length of the next line
              self.position = (self.position.0 + 1, next_column);
            }
            self.move_to(self.position.1, self.position.0)
          }
        }
      },
      Direction::Up => {
        // ensure we are not at the top of the file
        if self.view_frame.0 != 0 || self.position.0 != 0 {
          let file = &self.file;
          // get the row above current cursor
          if let Some(row) = file.get_row(self.position.0 as usize + self.view_frame.0 - 1) {
            // 'next_column' yet again ensures stickiness to the left
            let next_column = std::cmp::min(self.position.1, 1 + self.buffer + row.len() as u16);
            // at the top of the view frame, scroll one line up
            if self.position.0 == 0 {
              self.view_frame = (self.view_frame.0 - 1, self.view_frame.1 - 1);
              self.position = (self.position.0, next_column);
            } else { // stick left, move cursor left depending on the character length of the next line
              self.position = (self.position.0 - 1, next_column);
            }
            self.move_to(self.position.1, self.position.0)
          }
        }
      },
      Direction::Left => {
        if self.position.1 > self.buffer {
          if self.position.1 == self.buffer + 1 {
            if self.position.0 != 0 || self.view_frame.0 != 0 {
              self.scroll(Direction::Up);
              if let Some(row) = self.file
                .get_row(self.view_frame.0 + self.position.0 as usize) {
                  self.position.1 = std::cmp::min(self.terminal.width, self.buffer + 1 + row.len() as u16)
              }
            }
          } else {
            self.position.1 -= 1
          }
          self.move_to(self.position.1, self.position.0)
        }
      },
      Direction::Right => {
        if self.position.1 != self.terminal.width {
          if let Some(row) = self.file
            .get_row(self.view_frame.0 + self.position.0 as usize) {
              if row.len() + self.buffer as usize + 1 == self.position.1 as usize || row.len() == 0 {
                if self.view_frame.0 + self.position.0 as usize == self.file.len() - 1 {
                  return
                }
                self.scroll(Direction::Down);
                self.position.1 = self.buffer + 1;
              } else {
                self.position.1 += 1;
              }
          }
          self.move_to(self.position.1, self.position.0)
        }
      }
    }
  }

  fn set_cursor(&mut self, mode: &EditorMode) {
    let _ = execute!(
      stdout(),
      SetCursorShape(match *mode {
        EditorMode::Insert => CursorShape::Line,
        _ => CursorShape::Block
      })
    );
  }

  fn move_to_beginning(&mut self) {
    for _ in 0..self.file.len() {
      self.scroll(Direction::Up)
    }
  }

  fn move_to_next_word(&mut self) {
    let row = self.file.get_row(self.position.0 as usize + self.view_frame.0).unwrap();
    let mut slice = row.content()[(self.position.1 - self.buffer - 1) as usize..].chars().enumerate();
    while let Some((_, character)) = slice.next() {
      if character.is_whitespace() || character.is_ascii_punctuation() {
        while let Some((index, character)) = slice.next() {
          if !(character.is_whitespace() || character.is_ascii_punctuation()) {
            self.move_to(self.position.1 + index as u16, self.position.0);
            return
          }
        }
      }
    }
    self.move_to(self.buffer + 1, self.position.0 + 1)
  }

  fn move_to_prev_word(&mut self) {
    let row = self.file.get_row(self.position.0 as usize + self.view_frame.0).unwrap();
    let mut slice = row.content()[0..(self.position.1 - self.buffer) as usize].chars().rev().enumerate();
    while let Some((_, character)) = slice.next() {
      if character.is_whitespace() || character.is_ascii_punctuation() {
        while let Some((index, character)) = slice.next() {
          if !(character.is_whitespace() || character.is_ascii_punctuation()) {
            self.move_to(self.position.1 - index as u16, self.position.0);
            return
          }
        }
      }
    }
    self.move_to(self.buffer + 1, self.position.0 + 1)
  }

  fn move_to_end(&mut self) {
    for _ in self.view_frame.0..self.file.len() {
      self.scroll(Direction::Down)
    }
  }

  fn move_to_line_end(&mut self) {
    let row_len = self.file.get_row(self.position.0 as usize + self.view_frame.0).unwrap().len();
    self.move_to(row_len as u16 + self.buffer + 1, self.position.0)
  }

  fn move_to_line_beginning(&mut self) {
    self.move_to(self.buffer + 1, self.position.0)
  }

  fn clear_row(&self) {
    let _ = execute!(
      stdout(),
      Clear(ClearType::CurrentLine)
    );
  }

  fn get_file_mut(&mut self) -> &mut Document {
    &mut self.file
  }

  fn insert(&mut self, key: char) {
    let line = self.view_frame.0 + self.position.0 as usize;
    let column = self.position.1 - self.buffer - 1;
    {
      let rows = &self.file.rows[line..(line + 1)];
      self.history.push(HistoryNode::create(rows, line..(line + 1)));
    }
    let file = self.get_file_mut();
    let row = file.get_row_mut(line).unwrap();
    row.insert(if column as usize == row.len() {
      IPositionDescriptor::End(key)
    } else {
      IPositionDescriptor::Middle(column as usize, key)
    });
    self.scroll(Direction::Right)
  }

  fn delete(&mut self) {
    let line = self.view_frame.0 + self.position.0 as usize;
    let column = self.position.1 - self.buffer - 1;
    {
      let rows = &self.file.rows[line..(line + 1)];
      self.history.push(HistoryNode::create(rows, line..(line + 1)));
    }
    let row_length = self.file.rows.get(line).unwrap().len();
    let file = self.get_file_mut();
    if let Some(offset) = file.handle_delete(if column == 0 {
      DPositionDescriptor::Beginning(line)
    } else if column as usize == row_length {
      DPositionDescriptor::End(line)
    } else {
      DPositionDescriptor::Middle(line, (column - 1) as usize)
    }) {
      self.set_buffer();
      if self.position.0 == 0 && column == 0 {
        self.scroll(Direction::Up);
        self.move_to(offset as u16 + self.buffer + 1, self.position.0)
      } else {
        self.move_to(offset as u16 + self.buffer + 1, self.position.0 - 1)
      }
    } else {
      self.scroll(Direction::Left)
    }
  }

  fn insert_row(&mut self) {
    let line = self.view_frame.0 + self.position.0 as usize;
    let column = self.position.1 - self.buffer - 1;
    let file = self.get_file_mut();
    let row = file.get_row_mut(line).unwrap();
    let offset = {
      let mut counter: usize = 0;
      for character in row.content().chars() {
        if character.is_whitespace() {
          counter += 1
        } else {
          break
        }
      }
      counter / 2 * 2
    };
    let mut add_closing_brace = false;
    if let Some(last_key) = row.content().trim_end().chars().last() {
      add_closing_brace = last_key == '{';
    }
    let new_row = row.add_new_line(if column == 0 {
      NLPositionDescriptor::Beginning
    } else if column as usize == row.len() - 1 {
      NLPositionDescriptor::End
    } else {
      NLPositionDescriptor::Middle((column - 1) as usize)
    });
    file.insert_row(line + 1, new_row);
    self.scroll(Direction::Down);
    if add_closing_brace {
      self.insert_row();
      for _ in 0..offset {
        self.insert(' ')
      }
      self.insert('}');
      self.scroll(Direction::Up);
      for _ in 0..(offset + 2) {
        self.insert(' ')
      }
    } else {
      self.move_to_line_start()
    }
    self.set_buffer();
  }

  fn move_to_line_start(&mut self) {
    self.move_to(self.buffer + 1, self.position.0)
  }

  fn undo(&mut self) {
    if let Some(node) = self.history.pop() {
      let (range, mut altered_rows) = node.extract();
      altered_rows.reverse();
      let rest_cursor = range.end;
      for row_no in range {
        self.file.replace(row_no, altered_rows.pop().unwrap())
      }
      self.goto_line(rest_cursor);
      self.move_to_line_end()
    }
  }

  fn search(&mut self, expr: String) {
    let (num_results, results) = self.file.search_for(&expr);
    if num_results > 0 {
      self._search_total = num_results;
      self._search_current = 1;
      self.set_mode(EditorMode::Search);
      self.search_results = Some(results.into_iter())
    }
  }

  fn render(&mut self) {
    let _ = execute!(
      stdout(),
      Hide,
      SavePosition,
      MoveTo(0, 0),
    );
    let num_rows = self.file.rows.len();
    let buffer = num_rows.to_string().chars().count();
    self.buffer = buffer as u16;
    if self.mode != EditorMode::Command {
      self.file.highlight();
    }
    for terminal_row_no in self.view_frame.0..(self.view_frame.1 - 1) {
      if terminal_row_no < num_rows {
        self.clear_row();
        let used = buffer - (terminal_row_no + 1).to_string().chars().count();
        self.write_row(terminal_row_no + 1, used, self.file.rows.get(terminal_row_no).unwrap());
      } else {
        self.write_empty_line();
      }
    }
    match self.mode {
      EditorMode::Search => self.status_bar.render(self._search_current, self._search_total),
      _ => self.status_bar.render(self.view_frame.0 + self.position.0 as usize + 1, self.file.len())
    }
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
    let _ = self.terminal.clear();
    if self.altered && self._quit {
      if let Err(why) = self.file.save() {
        eprintln!("{}", why)
      }
    }
    let _ = execute!(
      stdout(),
      ResetColor,
      LeaveAlternateScreen,
      ResetColor,
      SetCursorShape(CursorShape::Block),
      EnableBlinking
    );
    let _ = disable_raw_mode();
    // println!("{:?}", self.file.highlighted_rows())
    // println!("{:?}", self.file)
    // println!("row {} column {}", self.position.0, self.position.1);
  }
}

fn numeric_modifer(commands: &mut String) -> u32 {
  let mut modifier = String::new();
  while let Some(character) = commands.pop() {
    if let Some(_) = character.to_digit(10) {
      modifier.push(character)
    } else {
      commands.push(character);
      break
    }
  }
  if let Ok(modifier) = modifier.parse::<u32>() {
    modifier
  } else {
    1
  }
}

fn word_modifier(commands: &mut String) -> Option<String> {
  let mut modifier = String::new();
  while let Some(character) = commands.pop() {
    modifier.push(character)
  }
  if modifier.len() > 0 {
    Some(modifier)
  } else {
    None
  }
}

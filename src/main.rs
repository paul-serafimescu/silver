mod editor;
mod file;
mod history;
mod highlighting;

use editor::Editor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = std::env::args().collect();
  Editor::new(args.get(1))?.run()?;
  Ok(())
}

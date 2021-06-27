mod editor;
mod file;

use editor::Editor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = std::env::args().collect();
  let _editor = Editor::new(args.get(1))?.run()?;

  Ok(())
}

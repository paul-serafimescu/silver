mod editor;
mod file;

use editor::Editor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = std::env::args().collect();
  let _editor = if let Some(document) = args.get(1) {
    Editor::new(document)?
  } else {
    Editor::default()?
  }.run()?;

  Ok(())
}

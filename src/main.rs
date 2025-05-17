mod app;
mod ui;
mod components;

use crate::app::App;
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use crossterm::execute;
use std::io::{stdout, Result};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.run(&mut terminal).await.expect("failed to run");

    disable_raw_mode()?;
    execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;

    Ok(())
}

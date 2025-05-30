mod app;
mod components;
mod config;
mod patroni;
mod services;
mod system;
mod ui;

use crate::app::App;
use crate::config::Config;
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{Result, stdout};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let config = Config::new();

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(config);
    app.run(&mut terminal).await.expect("failed to run");

    disable_raw_mode()?;
    execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;

    Ok(())
}

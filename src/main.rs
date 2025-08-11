// src/main.rs

mod app;
mod command;
mod completion;
mod error;
mod event;
mod state;
mod ui;

use app::App;
use crossterm::{
    cursor::Show,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use std::io;

use crate::error::AppResult;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Setup terminal with a guard to always restore state
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    struct TerminalCleanupGuard;
    impl Drop for TerminalCleanupGuard {
        fn drop(&mut self) {
            let _ = disable_raw_mode();
            let mut stdout = io::stdout();
            let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture, Show);
        }
    }

    let _guard = TerminalCleanupGuard;

    // Create and run the application
    let mut app = App::new()?;
    if let Err(err) = app.run(&mut terminal).await {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

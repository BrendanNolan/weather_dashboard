use crossterm::terminal;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

pub fn prepare_terminal_for_app_exit(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    terminal::disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}

pub fn create_terminal(
) -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn std::error::Error>> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

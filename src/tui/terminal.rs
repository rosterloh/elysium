use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal
};
use std::io::{self, Stderr};
use std::panic;

use super::event::EventHandler;
use super::{app::{App, AppResult}, ui};

pub type CrosstermTerminal = Terminal<CrosstermBackend<Stderr>>;

pub struct UserInterface {
    terminal: CrosstermTerminal,
    pub events: EventHandler,
}

impl UserInterface {
    pub fn new(terminal: CrosstermTerminal, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    pub fn draw(&mut self, state: &mut App) -> AppResult<()> {
        self.terminal.draw(|frame| ui::render(frame, state))?;
        Ok(())
    }

    pub fn enter(&mut self) -> AppResult<()> {
        ratatui::crossterm::terminal::enable_raw_mode()?;
        ratatui::crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;

        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    pub fn exit(&mut self) -> AppResult<()> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    fn reset() -> AppResult<()> {
        ratatui::crossterm::terminal::disable_raw_mode()?;
        ratatui::crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }
}
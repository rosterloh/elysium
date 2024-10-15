/// Application state handler.
mod app;
pub use self::app::App;
pub use self::app::AppResult;

/// Possible commands.
pub mod command;

/// Application configuration.
mod config;
pub use self::config::Config;

/// Terminal events handler.
pub mod event;

/// Widget renderer.
mod ui;

/// Custom widgets.
pub mod widgets;

use event::EventHandler;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}
    },
    Terminal
};
// use std::sync::atomic::Ordering;
use std::{io, panic};

/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal,
/// initialising the interface and handling the draw events.
#[derive(Debug)]
pub struct Tui<B: Backend> {
    /// Interface to the Terminal.
    terminal: Terminal<B>,
    /// Terminal event handler.
    pub events: EventHandler,
    // /// Is the interface paused?
    // pub paused: bool,
}

impl<B: Backend> Tui<B> {
    /// Constructs a new instance of [`Tui`].
    pub fn new(terminal: Terminal<B>, events: EventHandler) -> Self {
        Self {
            terminal,
            events,
            // paused: false,
        }
    }

    /// Initialises the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    pub fn init(&mut self) -> AppResult<()> {
        terminal::enable_raw_mode()?;
        ratatui::crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            better_panic::Settings::auto()
                .most_recent_first(false)
                .lineno_suffix(true)
                .create_panic_handler()(panic);
            std::process::exit(1);
        }));
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: tui::Terminal::draw
    /// [`rendering`]: crate::ui:render
    pub fn draw(&mut self, app: &mut App) -> AppResult<()> {
        self.terminal.draw(|frame| ui::render(frame, app))?;
        Ok(())
    }

    /// Toggles the [`paused`] state of interface.
    ///
    /// It disables the key input and exits the
    /// terminal interface on pause (and vice-versa).
    ///
    /// [`paused`]: Tui::paused
    // pub fn toggle_pause(&mut self) -> AppResult<()> {
    //     self.paused = !self.paused;
    //     if self.paused {
    //         Self::reset()?;
    //     } else {
    //         self.init()?;
    //     }
    //     self.events
    //         .key_input_disabled
    //         .store(self.paused, Ordering::Relaxed);
    //     Ok(())
    // }

    /// Reset the terminal interface.
    ///
    /// It disables the raw mode and reverts back the terminal properties.
    pub fn reset() -> AppResult<()> {
        terminal::disable_raw_mode()?;
        ratatui::crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        Terminal::new(CrosstermBackend::new(io::stdout()))?.show_cursor()?;
        Ok(())
    }

    /// Exits the terminal interface.
    ///
    /// It disables the raw mode and reverts back the terminal properties.
    pub fn exit(&mut self) -> AppResult<()> {
        terminal::disable_raw_mode()?;
        ratatui::crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        self.terminal.show_cursor()?;
        self.events.stop();
        Ok(())
    }
}
use std::error;

/// CLI arguments
pub mod args;

/// AWS data and functions
pub mod aws;

/// User interface.
pub mod tui;

/// Event, keybind, and commands
// pub mod handler;

/// App
// pub mod app;

/// Application result type.
pub type AppResult<T> = anyhow::Result<T, Box<dyn error::Error>>;
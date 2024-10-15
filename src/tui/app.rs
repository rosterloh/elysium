use std::{usize, error};
use std::sync::mpsc;

use crate::aws::AwsCloud;
use crate::tui::command::*;
use crate::tui::config::Config;
use crate::tui::event::Event;
use crate::tui::ui::{Tab, MAIN_TABS};
use crate::tui::widgets::list::SelectableList;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

/// Application result type.
pub type AppResult<T> = anyhow::Result<T, Box<dyn error::Error>>;

/// Application state.
#[derive(Debug)]
pub struct App {
    /// AWS Cloud interface.
    pub aws: AwsCloud,
    /// Current app configuration.
    pub cfg: Config,
    /// Selected tab.
    pub tab: Tab,
    /// Selected block.
    pub block_index: usize,
    /// List items.
    pub list: SelectableList<Vec<String>>,
    /// Show details.
    pub show_details: bool,
    /// Input.
    pub input: Input,
    /// Enable input.
    pub input_mode: bool,
    /// Device list scroll index.
    pub devices_scroll_index: usize,
    /// Should the application quit?
    pub should_quit: bool,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(aws: AwsCloud, cfg: Config) -> AppResult<Self> {
        let mut app = Self {
            aws,
            cfg,
            tab: Tab::default(),
            block_index: 0,
            list: SelectableList::default(),
            show_details: false,
            input: Input::default(),
            input_mode: false,
            devices_scroll_index: 0,
            should_quit: false,
        };
        app.handle_tab()?;
        Ok(app)
    }

    /// Runs a command and updates the state.
    pub fn run_command(
        &mut self,
        command: Command,
        _event_sender: mpsc::Sender<Event>,
    ) -> AppResult<()> {
        match command {
            Command::Input(command) => {
                match command {
                    InputCommand::Handle(event) => {
                        self.input.handle_event(&event);
                    }
                    InputCommand::Enter => {
                        self.input_mode = true;
                    }
                    InputCommand::Confirm => {
                        self.input_mode = false;
                    }
                    InputCommand::Resume(event) => {
                        if !self.input.value().is_empty() {
                            self.input_mode = true;
                            self.input.handle_event(&event);
                        }
                    }
                    InputCommand::Exit => {
                        self.input = Input::default();
                        self.input_mode = false;
                    }
                }
                self.handle_tab()?;
            }
            Command::ShowDetails => {
                self.show_details = !self.show_details;
            }
            Command::Next(scroll_type, amount) => match scroll_type {
                ScrollType::Tab => {
                    self.tab = (((self.tab as usize).checked_add(amount).unwrap_or_default())
                        % MAIN_TABS.len())
                    .into();
                    self.handle_tab()?;
                }
                ScrollType::Table => {
                    self.devices_scroll_index = self.devices_scroll_index.saturating_add(amount);
                }
                ScrollType::List => {
                    self.list.next(amount)
                }
                ScrollType::Block => {
                    self.block_index = (self.block_index.saturating_add(1)) % 3;
                }
            }
            Command::Previous(scroll_type, amount) => match scroll_type {
                ScrollType::Tab => {
                    self.tab = (self.tab as usize)
                        .checked_sub(amount)
                        .unwrap_or(MAIN_TABS.len() - 1)
                        .into();
                    self.handle_tab()?;
                }
                ScrollType::Table => {
                    self.devices_scroll_index = self.devices_scroll_index.saturating_sub(amount);
                }
                ScrollType::List => {
                    self.list.previous(amount)
                }
                ScrollType::Block => {
                    self.block_index = self.block_index.checked_sub(1).unwrap_or(0);
                }
            }
            Command::Top => {
                self.list.first();
            }
            Command::Bottom => {
                self.list.last();
            }
            Command::Increment => {
                // Do nothing.
            }
            Command::Decrement => {
                // Do nothing.
            }
            Command::Exit => {
                if self.show_details {
                    self.show_details = false;
                } else {
                    self.should_quit = true;
                }
            }
            Command::Nothing => {}
        }
        Ok(())
    }

    /// Update the state based on selected tab.
    pub fn handle_tab(&mut self) -> AppResult<()> {
        match self.tab {
            Tab::CoreDevices => {
                self.list = SelectableList::with_items(
                    self.aws
                        .devices
                        .clone()
                        .iter()
                        .map(|i| vec![i.name.to_string(), i.status.to_string(), i.last_status_update_timestamp.to_string()])
                        .filter(|items| {
                            self.input.value().is_empty()
                                || items.iter().any(|item| {
                                    item.to_lowercase()
                                        .contains(&self.input.value().to_lowercase())
                                })
                        })
                        .collect(),
                );
            }
            Tab::ThingGroups => {}
            Tab::Deployments => {}
        }
        Ok(())
    }

    /// Returns the key bindings.
    pub fn get_key_bindings(&self) -> Vec<(&str, &str)> {
        // match self.tab {
        //     Tab::CoreDevices => {
        vec![
            ("Enter", "Details"),
            ("/", "Search"),
            ("h/j/k/l", "Scroll"),
            ("n/p", "Toggle"),
            ("Tab", "Next"),
            ("q", "Quit"),
        ]
        //  }
    }

    // /// Changes the tab
    // pub fn set_tab(&mut self, tab: Tab) {
    //     self.tab = tab;
    // }

    // pub fn quit(&mut self) {
    //     self.should_quit = true;
    // }
}

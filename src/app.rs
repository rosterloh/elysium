use std::collections::HashMap;

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::Rect;
use tokio::sync::mpsc;
use tracing::debug;

use crate::{
    action::Action,
    aws::AwsCloud,
    components::{
        data_table::DataTable,
        header::Header,
        top_left::TopLeft,
        top_right::TopRight,
        Component
    },
    tui::{Event, Tui},
};

/// Application state.
pub struct App {
    tick_rate: f64,
    frame_rate: f64,
    components: Vec<Box<dyn Component>>,
    should_quit: bool,
    should_suspend: bool,
    mode: Mode,
    last_tick_key_events: Vec<KeyEvent>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    keybindings: KeyBindings,
    post_exist_msg: Option<String>,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Mode {
    #[default]
    Normal,
    Input,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum StateKey {
    Exact(Mode, KeyCode, KeyModifiers),
    KeyCode(KeyCode, KeyModifiers),
    State(Mode),
}

pub struct KeyBindings {
    map: HashMap<StateKey, Action>
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            map: [
                // Close app
                (
                    StateKey::Exact(Mode::Normal, KeyCode::Char('q'), KeyModifiers::empty()),
                    Action::Quit,
                ),
                (
                    StateKey::Exact(Mode::Normal, KeyCode::Char('d'), KeyModifiers::CONTROL),
                    Action::Quit,
                ),
                (
                    StateKey::Exact(Mode::Normal, KeyCode::Char('c'), KeyModifiers::CONTROL),
                    Action::Quit,
                ),
                // Suspend
                (
                    StateKey::Exact(Mode::Normal, KeyCode::Char('z'), KeyModifiers::CONTROL),
                    Action::Suspend,
                ),
                // Enter input mode
                (
                    StateKey::Exact(Mode::Normal, KeyCode::Char('i'), KeyModifiers::empty()),
                    Action::ModeChange(Mode::Input),
                ),
                // Exit input mode
                (
                    StateKey::Exact(Mode::Input, KeyCode::Esc, KeyModifiers::empty()),
                    Action::ModeChange(Mode::Normal),
                ),
                // Navigate with arrows or tab
                (
                    StateKey::Exact(Mode::Normal, KeyCode::Left, KeyModifiers::empty()),
                    Action::Left,
                ),
                (
                    StateKey::Exact(Mode::Normal, KeyCode::Right, KeyModifiers::empty()),
                    Action::Right,
                ),
                (
                    StateKey::Exact(Mode::Normal, KeyCode::Up, KeyModifiers::empty()),
                    Action::Up,
                ),
                (
                    StateKey::Exact(Mode::Normal, KeyCode::Down, KeyModifiers::empty()),
                    Action::Down,
                ),
                (
                    StateKey::Exact(Mode::Normal, KeyCode::Tab, KeyModifiers::empty()),
                    Action::Tab,
                ),
                // Toggle graph
                (
                    StateKey::Exact(Mode::Normal, KeyCode::Char('g'), KeyModifiers::empty()),
                    Action::GraphToggle,
                ),
                // Clear input
                (
                    StateKey::Exact(Mode::Normal, KeyCode::Char('c'), KeyModifiers::empty()),
                    Action::Clear,
                ),
            ]
            .into_iter()
            .collect(),
        }
    }
}

impl KeyBindings {
    pub fn get_action(&self, mode: Mode, key_event: KeyEvent) -> Option<&Action> {
        self.map
            .get(&StateKey::Exact(mode, key_event.code, key_event.modifiers))
            .or(self
                .map
                .get(&StateKey::KeyCode(key_event.code, key_event.modifiers)))
            .or(self.map.get(&StateKey::State(mode)))
    }
}

impl App {
    pub fn new(aws: AwsCloud) -> Result<Self> {
        let data_table = DataTable::new(aws);
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        Ok(Self {
            tick_rate: 1.0,
            frame_rate: 10.0,
            components: vec![
                Box::new(Header::new()),
                Box::new(TopLeft::default()),
                Box::new(TopRight::default()),
                Box::new(data_table)
            ],
            should_quit: false,
            should_suspend: false,
            mode: Mode::Normal,
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
            keybindings: KeyBindings::default(),
            post_exist_msg: None,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            // .mouse(true) // uncomment this line to enable mouse support
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(self.action_tx.clone())?;
        }
        for component in self.components.iter_mut() {
            component.init(tui.size()?)?;
        }

        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        
        if let Some(ref s) = self.post_exist_msg {
            println!("`elysium` failed with Error:");
            println!("{}", s);
        }

        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx.send(Action::Quit)?,
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,
            _ => {}
        }
        for component in self.components.iter_mut() {
            if let Some(action) = component.handle_events(Some(event.clone()))? {
                action_tx.send(action)?;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match self
                .keybindings
                .get_action(self.mode, key_event)
                .cloned()
                .map(|action| self.action_tx.send(action))
            {
                Some(Err(error)) => Err(error.into()),
                _ => Ok(())
            }
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
            }
            match action {
                Action::ModeChange(mode) => {
                    self.mode = mode;
                }
                Action::Error(ref err_msg) => {
                    self.post_exist_msg = Some(err_msg.to_string());
                    self.should_quit = true;
                }
                // Action::Export => {
                //     // get data from specific components by downcasting them and then try to
                //     // convert into specific struct
                //     for component in &self.components {
                //         if let Some(dt) = component.as_any().downcast_ref::<DataTable>() {
                //         } else if let Some(tl) = component.as_any().downcast_ref::<TopLeft>() {
                //         } else if let Some(tr) = component.as_any().downcast_ref::<TopRight>() {
                //         }
                //     }
                //     action_tx
                //         .send(Action::ExportData(ExportData {}))
                //         .unwrap();
                // }
                Action::Tick => {
                    self.last_tick_key_events.drain(..);
                }
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::Render => self.render(tui)?,
                _ => {}
            }
            for component in self.components.iter_mut() {
                if let Some(action) = component.update(action.clone())? {
                    self.action_tx.send(action)?
                };
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            for component in self.components.iter_mut() {
                if let Err(err) = component.draw(frame, frame.area()) {
                    let _ = self
                        .action_tx
                        .send(Action::Error(format!("Failed to draw: {:?}", err)));
                }
            }
        })?;
        Ok(())
    }
}

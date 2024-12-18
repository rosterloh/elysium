// use std::any::Any;

use color_eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::{Rect, Size};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    enums::TabsEnum,
    tui::{Event, Frame}
};

pub mod data_table;
pub mod header;
pub mod top_left;
pub mod top_right;

pub trait Component {
    #[allow(unused_variables)]
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        Ok(())
    }

    // #[allow(unused_variables)]
    // fn as_any(&self) -> &dyn Any;

    #[allow(unused_variables)]
    fn tab_changed(&mut self, tab: TabsEnum) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn init(&mut self, area: Size) -> Result<()> {
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let action = match event {
            Some(Event::Key(key_event)) => self.handle_key_events(key_event)?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event)?,
            _ => None,
        };
        Ok(action)
    }

    #[allow(unused_variables)]
    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }

    #[allow(unused_variables)]
    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        Ok(None)
    }

    #[allow(unused_variables)]
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()>;
}
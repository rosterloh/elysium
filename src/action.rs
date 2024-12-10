use crate::{app::Mode, enums::TabsEnum};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    #[allow(dead_code)]
    Help,

    Up,
    Down,
    Left,
    Right,
    Tab,
    TabChange(TabsEnum),
    ModeChange(Mode),
    GraphToggle,
    Clear,
    DataLoaded,
}
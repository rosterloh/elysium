use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};
use ratatui::{
    text::{Line, Span},
    widgets::Paragraph,
};
use strum::{EnumCount, IntoEnumIterator};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
    action::Action,
    enums::TabsEnum,
    layout::{get_vertical_layout, DEFAULT_BORDER_STYLE}
};

#[derive(Default)]
pub struct Header {
    action_tx: Option<UnboundedSender<Action>>,
    tab_index: usize,
}

impl Header {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            tab_index: 0,
        }
    }

    fn make_tabs(&self) -> Paragraph {
        let enum_titles: Vec<Span> =
            TabsEnum::iter()
                .enumerate()
                .fold(Vec::new(), |mut title_spans, (idx, p)| {
                    let mut s1 = format!("{} ", p).dark_gray().bold();
                    if idx == self.tab_index {
                        s1 = format!("{} ", p).green().bold();
                    }

                    title_spans.push(s1);
                    title_spans
                });

        let arrow = String::from(char::from_u32(0x25bc).unwrap_or('>'));
        let b = Block::default()
            // .title_top(Line::from(vec![
            //     "|".fg(Color::Rgb(100, 100, 100)),
            //     env!("CARGO_PKG_NAME").bold(),
            //     "-".fg(Color::Rgb(100, 100, 100)),
            //     env!("CARGO_PKG_VERSION").into(),
            //     "|".fg(Color::Rgb(100, 100, 100)),
            // ]).centered())
            .title_bottom(Line::from(vec!["|".yellow(), arrow.green(), "|".yellow()]).centered())
            .borders(Borders::ALL)
            .border_type(DEFAULT_BORDER_STYLE)
            .padding(Padding::new(1, 0, 0, 0))
            .border_style(Style::default().fg(Color::Rgb(100, 100, 100))); // Color::Cyan
            // .highlight_style(
            //     Style::default()
            //         .add_modifier(Modifier::BOLD)
            //         // .fg(self.config.highlight_style_fg),
            // );

        Paragraph::new(Line::from(enum_titles)).block(b)
    }

    fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % TabsEnum::COUNT;
        if let Some(ref action_tx) = self.action_tx {
            let tab_enum = TabsEnum::iter().nth(self.tab_index).unwrap();
            action_tx.send(Action::TabChange(tab_enum)).unwrap();
        }
    }
}

impl Component for Header {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    // fn as_any(&self) -> &dyn std::any::Any {
    //     self
    // }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tab => {
                self.next_tab();
            }

            Action::TabChange(tab_enum) => TabsEnum::iter().enumerate().for_each(|(idx, t)| {
                if tab_enum == t {
                    self.tab_index = idx;
                }
            }),

            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> Result<()> {
        let rect = Rect::new(0, 0, frame.area().width, 1);
        let pkg: &str = env!("CARGO_PKG_NAME");
        let version: &str = env!("CARGO_PKG_VERSION");
        let title = format!("{} - v{}", pkg, version);
        frame.render_widget(Paragraph::new(title), rect);

        let layout = get_vertical_layout(area);
        let mut rect = layout.tabs;
        rect.y += 1;

        let tabs = self.make_tabs();
        frame.render_widget(tabs, rect);

        Ok(())
    }
}
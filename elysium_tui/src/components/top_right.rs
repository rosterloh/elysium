use std::time::Instant;

use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    components::Component,
    layout::{get_horizontal_layout, get_vertical_layout, DEFAULT_BORDER_STYLE},
    tui::Frame,
};

pub struct TopRight {
    action_tx: Option<UnboundedSender<Action>>,
    last_update_time: Instant,
}

impl Default for TopRight {
    fn default() -> Self {
        Self::new()
    }
}

impl TopRight {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            last_update_time: Instant::now(),
        }
    }

    fn app_tick(&mut self) -> Result<()> {
        let now = Instant::now();
        let elapsed = (now - self.last_update_time).as_secs_f64();
        if elapsed > 5.0 {
            self.last_update_time = now;
        }
        Ok(())
    }

    fn make_table(&mut self) -> Table {
        let header = Row::new(vec!["", "", "", "", ""])
            .style(Style::default().fg(Color::Yellow))
            .height(1);
        let mut rows = Vec::new();
        rows.push(
            Row::new(vec![
                Cell::from(Span::styled(
                    "",
                    Style::default().fg(Color::Red),
                )),
                Cell::from(Span::styled(
                    "",
                    Style::default().fg(Color::Green),
                )),
                Cell::from(""),
                Cell::from(""),
                Cell::from(vec![Line::from("")]),
            ])
            .height(1)
        );
        let table = Table::new(
            rows,
            [
                Constraint::Length(1),
                Constraint::Length(8),
                Constraint::Length(18),
                Constraint::Length(14),
                Constraint::Length(25),
            ],
        )
        .header(header)
        .block(
            Block::default()
                // .title(Line::from(vec![
                //     Span::styled("|Inter", Style::default().fg(Color::Yellow)),
                //     Span::styled("f", Style::default().fg(Color::Red)),
                //     Span::styled("aces|", Style::default().fg(Color::Yellow)),
                // ]))
                .border_style(Style::default().fg(Color::Rgb(100, 100, 100)))
                .title_style(Style::default().fg(Color::Yellow))
                .title_alignment(Alignment::Right)
                .borders(Borders::ALL)
                .border_type(DEFAULT_BORDER_STYLE)
                .padding(Padding::new(0, 0, 1, 0)),
        )
        .column_spacing(1);
        table
    }
}

impl Component for TopRight {
    fn init(&mut self, _area: Size) -> Result<()> {
        Ok(())
    }

    // fn as_any(&self) -> &dyn std::any::Any {
    //     self
    // }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::Tick = action {
            self.app_tick()?
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let v_layout = get_vertical_layout(area);
        let h_layout = get_horizontal_layout(area);

        let table_rect = Rect::new(
            h_layout.right.x,
            1,
            h_layout.right.width,
            v_layout.top.height,
        );

        let block = self.make_table();
        f.render_widget(block, table_rect);

        Ok(())
    }
}
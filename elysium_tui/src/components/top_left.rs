use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    components::Component,
    layout::{get_horizontal_layout, get_vertical_layout, DEFAULT_BORDER_STYLE},
    tui::Frame,
};

#[derive(Debug)]
pub struct GraphDataset {
    name: String,
    data: Vec<(f64, f64)>,
    color: Color,
}

pub struct TopLeft {
    action_tx: Option<UnboundedSender<Action>>,
    show_graph: bool,
    graph_datasets: Vec<GraphDataset>,
    graph_tick: [f64; 2],
}

impl Default for TopLeft {
    fn default() -> Self {
        Self::new()
    }
}

impl TopLeft {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            show_graph: false,
            graph_datasets: Vec::new(),
            graph_tick: [0.0, 40.0],
        }
    }

    fn make_chart(&mut self) -> Chart {
        let mut datasets = Vec::new();
        for d in &self.graph_datasets {
            let dataset = Dataset::default()
                .name(&*d.name)
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(d.color))
                .graph_type(GraphType::Line)
                .data(&d.data);
            datasets.push(dataset);
        }

        let x_labels: Vec<Span> = [
            self.graph_tick[0].to_string(),
            (((self.graph_tick[1] - self.graph_tick[0]) / 2.0) + self.graph_tick[0]).to_string(),
            self.graph_tick[1].to_string(),
        ]
        .iter()
        .cloned()
        .map(Span::from)
        .collect();

        let chart = Chart::new(datasets)
            .block(
                Block::new()
                    .title_top(Line::from("|Chart Title|".yellow()).right_aligned())
                    .title_bottom(Line::from(vec![
                            Span::styled("|hide ", Style::default().fg(Color::Yellow)),
                            Span::styled("g", Style::default().fg(Color::Red)),
                            Span::styled("raph|", Style::default().fg(Color::Yellow)),
                        ]).right_aligned()
                    )
                    .border_style(Style::default().fg(Color::Rgb(100, 100, 100)))
                    .borders(Borders::ALL)
                    .border_type(DEFAULT_BORDER_STYLE)
                    .padding(Padding::new(1, 1, 1, 1)),
            )
            .y_axis(
                Axis::default()
                    .bounds([25.0, 100.0])
                    .title("[signal(dbm)]")
                    .labels(
                        ["-25.0", "-52.0", "-100.0"]
                            .iter()
                            .cloned()
                            .map(Span::from)
                            .collect::<Vec<Span>>(),
                    )
                    .style(Style::default().fg(Color::Yellow)),
            )
            .x_axis(
                Axis::default()
                    .bounds(self.graph_tick)
                    .title("[scans]")
                    .labels(x_labels)
                    .style(Style::default().fg(Color::Yellow)),
            )
            .legend_position(Some(LegendPosition::TopLeft))
            .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)));
        chart        
    }
}

impl Component for TopLeft {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }
    
    // fn as_any(&self) -> &dyn std::any::Any {
    //     self
    // }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::GraphToggle = action {
            self.show_graph = !self.show_graph;
        }

        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        if self.show_graph {
            let v_layout = get_vertical_layout(area);
            let h_layout = get_horizontal_layout(area);

            let rect = Rect::new(h_layout.left.x, 1, h_layout.left.width, v_layout.top.height);

            let block = self.make_chart();
            f.render_widget(block, rect);
        }
        Ok(())
    }
}
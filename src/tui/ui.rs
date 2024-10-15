use crate::{aws::Info, tui::app::App};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Position, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState, Tabs,
    },
    Frame,
};
use tui_input::Input;
use unicode_width::UnicodeWidthStr;

/// Titles of the main tabs.
pub const MAIN_TABS: &[&str] = Tab::get_headers();

/// Titles of the ELF info tabs.
pub const AWS_INFO_TABS: &[Info] = &[
    Info::CoreDevices,
    Info::ThingGroups,
    Info::Deployments,
];

/// Maximum number of elements to show in table/list.
const LIST_LIMIT: usize = 100;

/// Application tab.
#[derive(Clone, Copy, Debug, Eq, PartialEq, clap::ValueEnum)]
pub enum Tab {
    /// Greengrass Core Devices.
    CoreDevices = 0,
    /// IoT Core Thing Groups.
    ThingGroups = 1,
    /// Greengrass Deployments.
    Deployments = 2,
}

impl Default for Tab {
    fn default() -> Self {
        Self::CoreDevices
    }
}

impl Tab {
    /// Returns the available tabs.
    const fn get_headers() -> &'static [&'static str] {
        &["Core Devices", "Thing Groups", "Deployments"]
    }
}

impl From<usize> for Tab {
    fn from(v: usize) -> Self {
        match v {
            0 => Self::CoreDevices,
            1 => Self::ThingGroups,
            2 => Self::Deployments,
            _ => Self::default(),
        }
    }
}

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::new(
        Direction::Vertical,
        [Constraint::Length(3), Constraint::Min(0)],
    )
    .direction(Direction::Vertical)
    .margin(1)
    .split(f.area());

    {
        f.render_widget(
            Block::bordered()
                .title(vec![
                    "|".fg(Color::Rgb(100, 100, 100)),
                    env!("CARGO_PKG_NAME").bold(),
                    "-".fg(Color::Rgb(100, 100, 100)),
                    env!("CARGO_PKG_VERSION").into(),
                    "|".fg(Color::Rgb(100, 100, 100)),
                ])
                .title_alignment(Alignment::Center),
            chunks[0],
        );
        let chunks = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .margin(1)
        .split(chunks[0]);
        let tabs = Tabs::new(MAIN_TABS.iter().map(|v| Line::from(*v)))
            .select(app.tab as usize)
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(app.cfg.highlight_style_fg),
            );
        f.render_widget(tabs, chunks[0]);
    }
    match app.tab {
        Tab::CoreDevices => {
            render_device_info(app, f, chunks[1]);
        }
        Tab::ThingGroups => {
            render_group_info(app, f, chunks[1]);
        }
        Tab::Deployments => {
            // render_deployment_info(app, f, chunks[1]);
            render_device_info(app, f, chunks[1]);
        }
    }
    render_key_bindings(app, f, chunks[1]);
}

/// Renders the key bindings.
pub fn render_key_bindings(app: &mut App, frame: &mut Frame, rect: Rect) {
    let chunks = Layout::vertical([Constraint::Percentage(100), Constraint::Min(1)]).split(rect);
    let key_bindings = app.get_key_bindings();
    let line = Line::from(
        key_bindings
            .iter()
            .enumerate()
            .flat_map(|(i, (keys, desc))| {
                vec![
                    "[".fg(Color::Rgb(100, 100, 100)),
                    keys.yellow(),
                    "→ ".fg(Color::Rgb(100, 100, 100)),
                    Span::from(*desc),
                    "]".fg(Color::Rgb(100, 100, 100)),
                    if i != key_bindings.len() - 1 { " " } else { "" }.into(),
                ]
            })
            .collect::<Vec<Span>>(),
    );
    if line.width() as u16 > chunks[1].width.saturating_sub(25)
        && get_input_line(app).width() != 0
        // && (app.tab != Tab::ThingGroups || app.tab != Tab::Deployments)
    {
        return;
    }
    frame.render_widget(Paragraph::new(line.alignment(Alignment::Center)), chunks[1]);
}

/// Render devices info.
pub fn render_device_info(app: &mut App, frame: &mut Frame, rect: Rect) {
    let selected_index = app.list.state.selected().unwrap_or_default();
    let items_len = app.list.items.len();
    let page = selected_index / LIST_LIMIT;
    let headers = AWS_INFO_TABS[app.info_index].headers();
    let mut table_state = TableState::default();
    table_state.select(Some(selected_index % LIST_LIMIT));
    let max_row_width = (rect.width as usize / headers.len()).saturating_sub(2);
    let items = app
        .list
        .items
        .iter()
        .skip(page * LIST_LIMIT)
        .take(LIST_LIMIT)
        .map(|items| {
            Row::new(items.iter().enumerate().map(|(i, value)| {
                Cell::from(Line::from(if value.width() > max_row_width && i == 0 {
                    let mut spans = highlight_search_result(
                        value.chars().take(max_row_width).collect::<String>().into(),
                        &app.input,
                    );
                    spans.push("…".fg(Color::Rgb(100, 100, 100)));
                    spans
                } else {
                    highlight_search_result(value.to_string().into(), &app.input)
                }))
            }))
        });
    frame.render_stateful_widget(
        Table::new(
            items,
            &[Constraint::Percentage(
                (100 / headers.len()).try_into().unwrap_or_default(),
            )]
            .repeat(headers.len()),
        )
        .header(Row::new(
            headers.to_vec().iter().map(|v| Cell::from((*v).bold())),
        ))
        .block(
            Block::bordered()
                .border_style(Style::default().fg(if app.block_index == 0 {
                    Color::Yellow
                } else {
                    Color::Rgb(100, 100, 100)
                }))
                .title_bottom(
                    if items_len != 0 {
                        Line::from(vec![
                            "|".fg(Color::Rgb(100, 100, 100)),
                            format!("{}/{}", selected_index.saturating_add(1), items_len)
                                .fg(app.cfg.highlight_style_fg)
                                .bold(),
                            "|".fg(Color::Rgb(100, 100, 100)),
                        ])
                    } else {
                        Line::default()
                    }
                    .right_aligned(),
                )
                .title_bottom(get_input_line(app)),
        )
        .highlight_style(Style::default().fg(Color::Green)),
        rect,
        &mut table_state,
    );
    render_cursor(app, rect, frame);
   
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        rect.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut ScrollbarState::new(items_len).position(selected_index),
    );
    // render_details(app, rect, frame);
}

/// Render groups info.
pub fn render_group_info(_app: &mut App, _frame: &mut Frame, _rect: Rect) {
}

/// Render deployments info.
pub fn render_deployment_info(_app: &mut App, _frame: &mut Frame, _rect: Rect) {
}

/// Renders details popup.
// fn render_details(app: &mut App, area: Rect, frame: &mut Frame<'_>) {
//     if app.show_details {
//         let headers;
//         match app.tab {
//             Tab::CoreDevices => {
//                 headers = DEVICES_HEADERS;
//             }
//             _ => {
//                 unimplemented!()
//             }
//         }
//         let max_row_width = (area.width - 2) / 2;
//         let items = app.list.selected().cloned().unwrap_or_default();
//         let lines: Vec<Line> = items
//             .iter()
//             .enumerate()
//             .flat_map(|(i, v)| {
//                 let mut lines = Vec::new();
//                 if v.width() as u16 > max_row_width {
//                     lines.extend(
//                         textwrap::wrap(v, textwrap::Options::new(max_row_width as usize))
//                             .into_iter()
//                             .enumerate()
//                             .map(|(x, v)| {
//                                 if x == 0 {
//                                     Line::from(vec![
//                                         Span::styled(
//                                             headers[i].to_string(),
//                                             Style::default().fg(Color::Cyan),
//                                         ),
//                                         Span::raw(": ").fg(Color::Rgb(100, 100, 100)),
//                                         v.to_string().into(),
//                                     ])
//                                 } else {
//                                     Line::from(v.to_string())
//                                 }
//                             }),
//                     )
//                 } else {
//                     lines.push(Line::from(vec![
//                         Span::styled(headers[i].to_string(), Style::default().fg(Color::Cyan)),
//                         Span::raw(": ").fg(Color::Rgb(100, 100, 100)),
//                         Span::styled(v, Style::default().fg(app.cfg.highlight_style_fg)),
//                     ]));
//                 }
//                 lines
//             })
//             .collect();
//         let popup = Popup::new(Text::from(lines)).title(Line::from(vec![
//             "|".fg(Color::Rgb(100, 100, 100)),
//             "Details".fg(app.cfg.highlight_style_fg).bold(),
//             "|".fg(Color::Rgb(100, 100, 100)),
//         ]));
//         frame.render_widget(&popup, area);
//     }
// }

/// Renders the cursor.
fn render_cursor(app: &mut App, area: Rect, frame: &mut Frame<'_>) {
    if app.input_mode {
        let (x, y) = (
            area.x
                + Input::default()
                    .with_value(format!("search: {}", app.input.value()))
                    .visual_cursor() as u16
                + 2,
            area.bottom().saturating_sub(1),
        );
        frame.render_widget(
            Clear,
            Rect {
                x,
                y,
                width: 1,
                height: 1,
            },
        );
        frame.set_cursor_position(Position::new(x, y));
    }
}

/// Returns the input line.
fn get_input_line<'a>(app: &'a App) -> Line<'a> {
    if !app.input.value().is_empty() || app.input_mode {
        Line::from(vec![
            "|".fg(Color::Rgb(100, 100, 100)),
            "search: ".yellow(),
            app.input.value().fg(app.cfg.highlight_style_fg),
            if app.input_mode { " " } else { "" }.into(),
            "|".fg(Color::Rgb(100, 100, 100)),
        ])
    } else {
        Line::default()
    }
}

/// Returns the line with the search result highlighted.
fn highlight_search_result<'a>(line: Line<'a>, input: &'a Input) -> Vec<Span<'a>> {
    let line_str = line.to_string();
    if line_str.contains(input.value()) && !input.value().is_empty() {
        let splits = line_str.split(input.value());
        let chunks = splits.into_iter().map(|c| Span::from(c.to_owned()));
        let pattern = Span::styled(
            input.value(),
            Style::new().bg(Color::Yellow).fg(Color::Black),
        );
        itertools::intersperse(chunks, pattern).collect::<Vec<Span>>()
    } else {
        line.spans.clone()
    }
}
use std::sync::Arc;

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, Event};
use ratatui::{
    layout::{Constraint, Rect},
    prelude::*,
    style::{Style, Stylize},
    text::Span,
    widgets::*,
};
use tokio::{
    sync::{mpsc::UnboundedSender, Mutex},
    task::JoinHandle,
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{
    action::Action,
    app::Mode,
    aws::{AwsCloud, Property},
    components::Component,
    enums::TabsEnum,
    layout::{get_vertical_layout, DEFAULT_BORDER_STYLE},
    tui::Frame,
};

static INPUT_SIZE: usize = 30;
const SPINNER_SYMBOLS: [&str; 6] = ["⠷", "⠯", "⠟", "⠻", "⠽", "⠾"];

pub struct DataTable {
    // aws: Arc<RwLock<AwsCloud>>,
    aws: Arc<Mutex<AwsCloud>>,
    // aws: AwsCloud,
    active_tab: TabsEnum,
    action_tx: Option<UnboundedSender<Action>>,
    data_list: Vec<Vec<String>>,
    table_state: TableState,
    scrollbar_state: ScrollbarState,
    input: Input,
    is_loading: bool,
    task: JoinHandle<()>,
    mode: Mode,
    filter_str: String,
    spinner_index: usize,
}

impl DataTable {
    pub fn new(aws: AwsCloud) -> Self {
        // RwLock: often read but rarely write (https://docs.rs/tokio/latest/tokio/sync/struct.RwLock.html)
        // Mutex: update data on every read (https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html)
        // let aws = Arc::new(RwLock::new(aws));
        let aws = Arc::new(Mutex::new(aws));
        Self {
            aws: aws,
            active_tab: TabsEnum::Devices,
            action_tx: None,
            data_list: Vec::new(),
            table_state: TableState::default().with_selected(0),
            scrollbar_state: ScrollbarState::new(0),
            input: Input::default().with_value(String::from("")),
            is_loading: false,
            task: tokio::spawn(async {}),
            mode: Mode::Normal,
            filter_str: String::from(""),
            spinner_index: 0,
        }
    }

    fn reset_data(&mut self) {
        self.data_list.clear();
        // self.data_index = 0;
    }

    fn load_data(&mut self) {
        self.reset_data();

        self.is_loading = true;

        let tx = self.action_tx.clone().unwrap();
        let aws = self.aws.clone();

        self.task = tokio::spawn(async move {
            // let mut write = aws.write().await;
            // write.load().await.unwrap();
            // drop(write);
            aws.lock().await.load().await.unwrap();
            // tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
            tx.send(Action::DataLoaded).unwrap_or_default();
        });
    }

    fn set_scrollbar_height(&mut self) {
        let mut data_len = 0;
        if !self.data_list.is_empty() {
            data_len = self.data_list.len() - 1;
        }
        self.scrollbar_state = self.scrollbar_state.content_length(data_len);
    }

    fn previous_in_table(&mut self) {
        let index = match self.table_state.selected() {
            Some(index) => {
                if index == 0 {
                    if self.data_list.is_empty() {
                        0
                    } else {
                        self.data_list.len() - 1
                    }
                } else {
                    index - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(index));
        self.scrollbar_state = self.scrollbar_state.position(index);
    }

    fn next_in_table(&mut self) {
        let index = match self.table_state.selected() {
            Some(index) => {
                let mut s_ip_len = 0;
                if !self.data_list.is_empty() {
                    s_ip_len = self.data_list.len() - 1;
                }
                if index >= s_ip_len {
                    0
                } else {
                    index + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(index));
        self.scrollbar_state = self.scrollbar_state.position(index);
    }

    fn make_table(
        data_list: &Vec<Vec<String>>,
        is_loading: bool,
        filter_str: String,
    ) -> Table {
        let header = Row::new(vec!["Name", "Status", "Last Status Update",])
            .style(Style::default().fg(Color::Yellow))
            .top_margin(1)
            .bottom_margin(1);
        let mut rows = Vec::new();

        for data in data_list {
            let (name, status, timestamp) = (data[0].clone(), data[1].clone(), data[2].clone());

            if filter_str.is_empty() || (name.contains(&filter_str) && !filter_str.is_empty()) {
                rows.push(Row::new(vec![
                    Cell::from(Span::styled(
                        format!("{}", name),
                        Style::default().fg(Color::Blue),
                    )),
                    Cell::from(status.green()),
                    Cell::from(timestamp),
                    // Cell::from(sip.vendor.as_str().yellow()),
                ]));
            }
        }

        let mut loading_title = vec![
            Span::styled("|", Style::default().fg(Color::Yellow)),
            "◉ ".green(),
            Span::styled(
                format!("{}", data_list.len()),
                Style::default().fg(Color::Red),
            ),
            Span::styled("|", Style::default().fg(Color::Yellow)),
        ];
        if is_loading {
            loading_title.push(" ⣿(".yellow());
            loading_title.push(format!("{}", 0).red());
            loading_title.push(format!("/{}", 0).green());
            loading_title.push(")".yellow());
        }

        let table = Table::new(
            rows,
            [
                Constraint::Length(30),
                Constraint::Length(10),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ],
        )
        .header(header)
        .block(
            Block::new()
                // .title_top(
                //     Line::from("|Data|".yellow()).right_aligned()
                // )
                // .title_bottom(
                //     Line::from(vec![
                //         Span::styled("|", Style::default().fg(Color::Yellow)),
                //         Span::styled(
                //             "e",
                //             Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
                //         ),
                //         Span::styled("xport data", Style::default().fg(Color::Yellow)),
                //         Span::styled("|", Style::default().fg(Color::Yellow)),
                //     ]).left_aligned()
                // )
                .title_top(Line::from(loading_title).left_aligned())
                .title_bottom(
                    Line::from(vec![
                        Span::styled("|", Style::default().fg(Color::Yellow)),
                        String::from(char::from_u32(0x25b2).unwrap_or('>')).red(),
                        String::from(char::from_u32(0x25bc).unwrap_or('>')).red(),
                        Span::styled(" select|", Style::default().fg(Color::Yellow)),
                    ]).right_aligned()
                )
                .border_style(Style::default().fg(Color::Rgb(100, 100, 100)))
                .borders(Borders::ALL)
                .border_type(DEFAULT_BORDER_STYLE),
        )
        .highlight_symbol(String::from(char::from_u32(0x25b6).unwrap_or('>')).red())
        .column_spacing(1);
        table
    }

    pub fn make_scrollbar<'a>() -> Scrollbar<'a> {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(Color::Rgb(100, 100, 100)))
            .begin_symbol(None)
            .end_symbol(None);
        scrollbar
    }

    fn make_input(&self, scroll: usize) -> Paragraph {
        let input = Paragraph::new(self.input.value())
            .style(Style::default().fg(Color::Green))
            .scroll((0, scroll as u16))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(match self.mode {
                        Mode::Input => Style::default().fg(Color::Green),
                        Mode::Normal => Style::default().fg(Color::Rgb(100, 100, 100)),
                    })
                    .border_type(DEFAULT_BORDER_STYLE)
                    .title_bottom(Line::from(vec![
                        Span::raw("|"),
                        Span::styled(
                            "c",
                            Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
                        ),
                        Span::styled("lear", Style::default().fg(Color::Yellow)),
                        Span::raw("|"),
                    ]).left_aligned())
                    .title_bottom(Line::from(vec![
                        Span::raw("|"),
                        Span::styled(
                            "i",
                            Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
                        ),
                        Span::styled("nput", Style::default().fg(Color::Yellow)),
                        Span::raw("/"),
                        Span::styled(
                            "ESC",
                            Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
                        ),
                        Span::raw("|"),
                    ]).centered())
            );
        input
    }

    fn make_spinner(&self) -> Span {
        let spinner = SPINNER_SYMBOLS[self.spinner_index];
        Span::styled(
            format!("{spinner}loading.."),
            Style::default().fg(Color::Yellow),
        )
    }

    fn set_filter_str(&mut self, value: String) {
        self.filter_str = value;
    }
}

impl Component for DataTable {
    fn init(&mut self, _area: Size) -> Result<()> {
        self.load_data();
        Ok(())
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    // fn as_any(&self) -> &dyn std::any::Any {
    //     self
    // }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        // if self.active_tab == TabsEnum::Devices
        let action = match self.mode {
            Mode::Normal => return Ok(None),
            Mode::Input => match key.code {
                KeyCode::Enter => {
                    if let Some(_sender) = &self.action_tx {
                        self.set_filter_str(self.input.value().to_string());
                    }
                    Action::ModeChange(Mode::Normal)
                }
                _ => {
                    self.input.handle_event(&Event::Key(key));
                    return Ok(None);
                }
            },
        };
        Ok(Some(action))
        // } else {
        //     Ok(None)
        // }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if self.is_loading {
            if let Action::Tick = action {
                let mut s_index = self.spinner_index + 1;
                s_index %= SPINNER_SYMBOLS.len() - 1;
                self.spinner_index = s_index;
            }
        }

        if let Action::DataLoaded = action {
            self.is_loading = false;
            self.tab_changed(TabsEnum::Devices).unwrap();
        }

        if let Action::Down = action {
            self.next_in_table();
        }
        if let Action::Up = action {
            self.previous_in_table();
        }

        if let Action::ModeChange(mode) = action {
            if self.is_loading && mode == Mode::Input {
                self.action_tx
                    .clone()
                    .unwrap()
                    .send(Action::ModeChange(Mode::Normal))
                    .unwrap();
                return Ok(None);
            }
            self.mode = mode;
        }

        if let Action::TabChange(tab) = action {
            self.tab_changed(tab).unwrap();
        }

        if let Action::Clear = action {
            self.input.reset();
            self.filter_str = String::from("");
        }

        Ok(None)
    }

    fn tab_changed(&mut self, tab: TabsEnum) -> Result<()> {
        self.active_tab = tab;

        futures::executor::block_on(async {
            match tab {
                TabsEnum::Devices => self.data_list = self.aws.lock().await.devices.items(),
                TabsEnum::Deployments => self.data_list = self.aws.lock().await.deployments.items(),
            }
        });
        
        self.set_scrollbar_height();

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> Result<()> {
        let layout = get_vertical_layout(area);
        
        let mut table_rect = layout.bottom;
        table_rect.y += 1;
        table_rect.height -= 1;

        let table = Self::make_table(&self.data_list, self.is_loading, self.filter_str.clone());
        frame.render_stateful_widget(table, table_rect, &mut self.table_state);

        let scrollbar = Self::make_scrollbar();
        let mut scroll_rect = table_rect;
        scroll_rect.y += 3;
        scroll_rect.height -= 3;
        frame.render_stateful_widget(
            scrollbar,
            scroll_rect.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.scrollbar_state,
        );

        let input_size: u16 = INPUT_SIZE as u16;
        let input_rect = Rect::new(
            table_rect.width - (input_size + 1),
            table_rect.y + 1,
            input_size,
            3,
        );

        // -- INPUT_SIZE - 3 is offset for border + 1char for cursor
        let scroll = self.input.visual_scroll(INPUT_SIZE - 3);
        let mut block = self.make_input(scroll);
        if self.is_loading {
            block = block.add_modifier(Modifier::DIM);
        }
        frame.render_widget(block, input_rect);

        match self.mode {
            Mode::Input => {
                frame.set_cursor_position(Position {
                    x: input_rect.x
                        + ((self.input.visual_cursor()).max(scroll) - scroll) as u16
                        + 1,
                    y: input_rect.y + 1,
                });
            }
            Mode::Normal => {}
        }

        if self.is_loading {
            let throbber = self.make_spinner();
            let throbber_rect = Rect::new(input_rect.x + 1, input_rect.y, 12, 1);
            frame.render_widget(throbber, throbber_rect);
        }

        Ok(())
    }
}
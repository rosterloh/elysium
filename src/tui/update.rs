use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::{App, InputMode};

pub fn update(app: &mut App, key_event: KeyEvent) {
    match app.input_mode {
        InputMode::Select => match key_event.code {
            // Exit application on `Ctrl-C` or q
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.quit();
                }
            }
            KeyCode::Char('q') => app.quit(),
            KeyCode::Char('p') => {
                if app.items.get_selected().is_some() {
                    app.input_mode = InputMode::Preview;
                }
            }
            
            KeyCode::Enter => {
                if app.items.get_selected().is_some() {
                    app.task_to_exec = app.items.get_selected();
                    app.quit()
                }
                }
            KeyCode::Down | KeyCode::Char('j') => app.items.next(),
            KeyCode::Up | KeyCode::Char('k') => app.items.previous(),
            KeyCode::Char('/') => app.input_mode = InputMode::Search,
            _ => {}
        },
        InputMode::Search => match key_event.code {
            KeyCode::Char(c) => {
                app.search.push(c);
                app.items.filter(&app.search);
            }
            KeyCode::Backspace => {
                _ = app.search.pop();
                app.items.filter(&app.search);
            }
            KeyCode::Esc => {
                app.search = String::new();
                app.items.filter(&app.search);
                app.input_mode = InputMode::Select;
            }
            KeyCode::Enter => app.input_mode = InputMode::Select,
            _ => {}
        },
        InputMode::Preview => match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('p') => app.input_mode = InputMode::Select,
            _ => {}
        },
    }
}
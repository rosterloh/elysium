use std::env;

use ratatui::style::Color;
use std::str::FromStr;

const ENV_PREFIX: &str = "ELYSIUM_";

#[derive(Debug)]
pub struct Config {
    pub highlight_style_bg: Color,
    pub highlight_style_fg: Color,
}

impl Config {
    pub fn load() -> Config {
        Config {
            highlight_style_bg: env::var(ENV_PREFIX.to_string() + "HIGHLIGHT_STYLE_BG")
                .unwrap_or("".to_string())
                .parse()
                .unwrap_or(Color::from_str("#ffffff").unwrap()),
            highlight_style_fg: env::var(ENV_PREFIX.to_string() + "HIGHLIGHT_STYLE_FG")
                .unwrap_or("".to_string())
                .parse()
                .unwrap_or(Color::from_str("#4c4f69").unwrap()),
        }
    }
}
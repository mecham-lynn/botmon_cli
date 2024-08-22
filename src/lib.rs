use std::{io::{self, stdout, Stdout}, str::FromStr};

use crossterm::{execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{backend::CrosstermBackend, style::{Color, Style}, Terminal};

pub mod error;
pub mod app;
pub mod bot_stats;
pub mod dynamo;
pub mod ui;
pub mod pages;
pub mod leo_config;
pub mod app_params;


pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal window
pub fn init() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Ok(Terminal::new(CrosstermBackend::new(stdout()))?)
}

/// Restores the terminal to it's previous state
pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}


pub enum Bus {
    ProdBus,
    StagingBus,
    TestBus,
    ProdStreamsBus,
    StagingStreamsBus,
    TestStreamsBus,
    ProdChubBus,
    StagingChubBus,
    TestChubBus
}


impl FromStr for Bus {
    type Err = color_eyre::Report;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}


pub struct Theme {
    pub key_binding: KeyBinding,
}

pub struct KeyBinding {
    pub key: Style,
    pub description: Style,
}

pub const THEME: Theme = Theme {
    key_binding: KeyBinding {
        key: Style::new().fg(BLACK).bg(DARK_GRAY),
        description: Style::new().fg(DARK_GRAY).bg(BLACK),
    },
};

const BLACK: Color = Color::Rgb(8, 8, 8);
const DARK_GRAY: Color = Color::Rgb(68, 68, 68);


use std::{io::{self, stdout, Stdout}, path::Path, str::FromStr};

use argh::FromArgs;
use chrono::Duration;
use color_eyre::eyre::Error;
use crossterm::{execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{backend::CrosstermBackend, style::{Color, Style}, Terminal};

pub mod error;
pub mod app;
pub mod bot_stats;
pub mod dynamo;
pub mod ui;
pub mod pages;
pub mod leo_config;


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

#[derive(Debug, FromArgs)]
/// cli utility to get bot/queue stats and information
pub struct AppParams {
    /// time for the app to refresh for new stats in seconds
    /// the max amount of time is 10 minutes. The minimum is 10 seconds. If an invalid number is passed in 
    /// the duration will be set to 10 seconds 
    #[argh(option, short='r', from_str_fn(num_to_duration))]
    pub refresh_time: Duration,
    
    #[argh(option, short='c')]
    /// path to a file that contains the leo-config file. It should contain all the details needed to interact
    /// with the LeoBus in question
    pub config_path: Option<String>,
    
    #[argh(option, short='b')]
    /// the actual key for the bus from the configuration file. 
    /// If not provided a select screen will display where a bus can be chosen.
    pub bus: Option<String>,
    
}

fn num_to_duration(value: &str) -> Result<Duration, String> {
    match value.parse::<i64>() {
        Ok(secs) => if secs > 600 {
                Ok(Duration::seconds(600))
            } else if secs < 10 {
                Ok(Duration::seconds(10))
            } else {
                Ok(Duration::seconds(secs))
            }
        Err(_) => Ok(Duration::seconds(10))
    }
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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


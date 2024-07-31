use std::collections::HashMap;

use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyEvent};
use itertools::Itertools;
use ratatui::widgets::ScrollbarState;

use crate::{app::Navigate, leo_config::LeoConfig};

#[derive(Debug)]
pub struct BusSelectState {
    pub buses: Vec<String>,
    pub bus_selected_index: usize,
    pub vertical_scroll: usize,
    pub vertical_scroll_state: ScrollbarState,
}

impl BusSelectState {
    pub fn new(buses: &HashMap<String, LeoConfig>) -> Self {
        Self {
            buses: buses.keys().map(|a| a.clone()).sorted().collect(),
            bus_selected_index: 0,
            vertical_scroll: 0,
            vertical_scroll_state: ScrollbarState::default(),
        }
    }
}

// impl Navigate for BusSelectState {
//     fn navigate(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
//         match key_event.code {
//             KeyCode::Up => {
                
//                 self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
//                 self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
//             }
//             a => {
//                 bail!("invalid key {a:?} pressed");
//             }
//         }
//     }
// }
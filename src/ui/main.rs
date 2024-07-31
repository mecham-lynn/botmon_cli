use ratatui::{layout::Rect, style::{Modifier, Style, Stylize}, widgets::{Block, Borders, List, ListItem, ListState}, Frame};

use crate::app::AppState;

use super::center_rect;

pub fn main_ui(app: &mut AppState, area: Rect, frame: &mut Frame) {
    let area = center_rect(area, 50, 50);
    let items = [
        ListItem::new("Bot Details"),
        ListItem::new("Queue Details")
    ];
    
    let mut state = ListState::default()
        .with_selected(Some(app.tab_index));
    let list = List::new(items)
        .block(Block::default().title(app.selected_bus.as_ref().unwrap().as_str()).borders(Borders::ALL))
        .style(Style::new().white().on_black())
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>");
    
    frame.render_stateful_widget(list, area, &mut state);
}
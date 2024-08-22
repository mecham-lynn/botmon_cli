use ratatui::{layout::Rect, style::{Color, Modifier, Style, Stylize}, widgets::{Block, Borders, List, ListItem, ListState}, Frame};

use crate::pages::bus_select::BusSelectState;

use super::center_rect;

pub fn bus_select(state: &mut BusSelectState, area: Rect, frame: &mut Frame) {
    let area = center_rect(area, 50, 50);

    let items: Vec<ListItem> = state.buses.iter().map(|a| ListItem::new(a.clone())).collect();

    let mut state = ListState::default()
        .with_selected(Some(state.bus_selected_index));

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::new().white().on_black())
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED)
                .fg(Color::LightRed)
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, area, &mut state);

}
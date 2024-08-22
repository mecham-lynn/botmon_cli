use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style, Stylize}, widgets::{Block, Borders, List, ListItem, ListState, Paragraph}, Frame};

use crate::pages::queue::QueueSearchState;

use super::center_rect;

pub fn queue_select(state: &mut QueueSearchState, area: Rect, frame: &mut Frame) {
    let area = center_rect(area, 80, 80);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            &[
                Constraint::Length(4),
                Constraint::Length(3),
                Constraint::Min(4)
            ]
        )
        .split(area);
    
    let width = chunks[0].width.max(3) - 3;
    let scroll = state.search.visual_scroll(width as usize);
    
    let input = Paragraph::new(state.search.value())
        .style(Style::default().fg(Color::Yellow))
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title("Bot Select"));
    
    frame.render_widget(input, chunks[1]);
    
    frame.set_cursor(
        // Put the cursor past the end of the input text
        chunks[1].x
        + ((state.search.visual_cursor()).max(scroll) - scroll) as u16
        + 1,
        // Move one line down, from the border to the input line
        chunks[1].y + 1,
    );
    
    let items: Vec<ListItem> = state.search_results.iter().map(|a| ListItem::new(a.clone())).collect();
    
    let mut list_state = ListState::default()
        .with_selected(Some(state.current_select_index));
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::new().white().on_black())
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED)
                .fg(Color::LightRed)
        )
        .highlight_symbol(">>");
        frame.render_stateful_widget(list, chunks[2], &mut list_state);
    
}
use ratatui::{layout::Rect, widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation}, Frame};

use crate::app::AppState;

pub fn view_app_state(state: &mut AppState, area: Rect, frame: &mut Frame) {
    
    let settings = format!("{:#?}", state);
    let num_lines = settings.lines().count();

    if state.vertical_scroll >= num_lines - (area.height as usize - 5){
        state.stop_scroll = true;
    } else if state.stop_scroll{
        state.stop_scroll = false;
    }
    
    state.vertical_scroll_state = state
        .vertical_scroll_state.content_length(num_lines);
    
    let paragraph = Paragraph::new(settings)
        .block(Block::default().borders(Borders::ALL).title("APP STATE"))
        .scroll((state.vertical_scroll as u16, 0));
    
    frame.render_widget(paragraph, area);
    frame.render_stateful_widget(Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("^"))
        .end_symbol(Some("v")),
    area,
    &mut state.vertical_scroll_state);
    
}
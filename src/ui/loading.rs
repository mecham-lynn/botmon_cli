use ratatui::{layout::{Constraint, Direction, Layout, Rect}, Frame};

use crate::app::AppState;

use super::center_rect;

pub fn loading(app: &mut AppState, area: Rect, frame: &mut Frame) {
    let area = center_rect(area, 45, 45);
    // let chunks = Layout::default()
    //     .direction(Direction::Horizontal)
    //     .margin(1)
    //     .constraints(
    //         [
    //             Constraint::Percentage(50),
    //             Constraint::Percentage(50),
    //         ]
    //         .as_ref(),
    //     )
    //     .split(area);

    // Simple random step
    // let simple = throbber_widgets_tui::Throbber::default();
    // frame.render_widget(simple, chunks[0]);

    // Set full with state
    let full = throbber_widgets_tui::Throbber::default()
        .label("Running...")
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan))
        .throbber_style(ratatui::style::Style::default().fg(ratatui::style::Color::Red).add_modifier(ratatui::style::Modifier::BOLD))
        .throbber_set(throbber_widgets_tui::CLOCK)
        .use_type(throbber_widgets_tui::WhichUse::Spin);
    frame.render_stateful_widget(full, area, &mut app.throbber_state);
}
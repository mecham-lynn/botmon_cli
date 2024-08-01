use bot::{bot_search_and_select_ui, bot_view_ui};
use itertools::Itertools;
use loading::loading;
use main::main_ui;
use ratatui::{layout::{self, Constraint, Direction, Layout, Rect}, style::{Color, Stylize}, text::{Line, Span}, widgets::Paragraph, Frame};

use crate::{app::{AppState, AppTab}, THEME};
mod chart;
mod main;
mod bot;
mod bus_select;
mod loading;

pub fn render_ui(frame: &mut Frame, app: &mut AppState) {
    let area = center_rect(frame.size(), 95, 95);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);
    
    match app.mode {
        crate::app::AppTab::Main => main_ui(app, layout[0], frame),
        crate::app::AppTab::Bot => bot_search_and_select_ui(&mut app.bot_page, layout[0], frame),
        crate::app::AppTab::Queue => todo!(),
        crate::app::AppTab::BotView => match &mut app.bot_page.selected_bot {
            Some(bot) => bot_view_ui(bot, layout[0], frame),
            None => panic!("cannot view non-existant bot"),
        }
        AppTab::BusSelect => bus_select::bus_select(&mut app.bus_select, layout[0], frame),
        AppTab::Loading => loading(app, area, frame),
       }
    
    render_bottom_bar(&app.mode, layout[1], frame)
    // Split the area when we want to show other charts
    // render_executions(frame, area, app)
    
    
}


fn center_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
          Constraint::Percentage((100 - percent_y) / 2),
          Constraint::Percentage(percent_y),
          Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    
      Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
          Constraint::Percentage((100 - percent_x) / 2),
          Constraint::Percentage(percent_x),
          Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn render_bottom_bar(mode: &AppTab, area: Rect, f: &mut Frame) {
    let keys = mode.get_keys();
    let spans = keys
        .iter()
        .flat_map(|(key, desc)| {
            let key = Span::styled(format!(" {} ", key), THEME.key_binding.key);
            let desc = Span::styled(format!(" {} ", desc), THEME.key_binding.description);
            [key, desc]
        })
        .collect_vec();
    let paragraph = Paragraph::new(Line::from(spans))
        .alignment(layout::Alignment::Center)
        .fg(Color::Indexed(236));
        // .bg(Color::Indexed(232));
    f.render_widget(paragraph, area)
}

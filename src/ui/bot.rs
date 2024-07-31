
use color_eyre::eyre::Context;
use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::{self, Color, Modifier, Style, Stylize}, text::Text, widgets::{canvas, Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Scrollbar, ScrollbarOrientation, StatefulWidget, Table}, Frame};

use crate::{app::AppState, bot_stats::QueueStats, pages::bot::{BotPageState, BotViewState}};

use style::palette::tailwind;
use super::center_rect;

pub fn bot_search_and_select_ui(page_state: &mut BotPageState, area: Rect, frame: &mut Frame) {
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
    let scroll = page_state.search.visual_scroll(width as usize);
    
    let input = Paragraph::new(page_state.search.value())
        .style(Style::default().fg(Color::Yellow))
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title("Bot Select"));
    
    frame.render_widget(input, chunks[1]);
    
    frame.set_cursor(
        // Put the cursor past the end of the input text
        chunks[1].x
        + ((page_state.search.visual_cursor()).max(scroll) - scroll) as u16
        + 1,
        // Move one line down, from the border to the input line
        chunks[1].y + 1,
    );

    let items: Vec<ListItem> = page_state.search_results.iter().map(|a| ListItem::new(a.clone())).collect();
    
    let mut state = ListState::default()
        .with_selected(Some(page_state.current_select_index));
    
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
        frame.render_stateful_widget(list, chunks[2], &mut state);
    // }
    //TODO remove the below when the new view is ready
    // if let Some(name) = page_state.selected_bot_name.as_ref() {
    //     let selected_record = Paragraph::new(name.to_owned())
    //         .block(Block::default().borders(Borders::all()).title("selected"));
    //     frame.render_widget(selected_record, chunks[0])
    // }
}

pub fn bot_view_ui(state: &mut BotViewState, area: Rect, frame: &mut Frame) {
    
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(2)
        .constraints(
            &[
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
        )
        .split(area);
    
    let settings_json_pretty = serde_json::to_string_pretty(&state.setting).wrap_err("failed to make setting pretty json").unwrap();
    let num_lines = settings_json_pretty.lines().count();
    
    state.vertical_scroll_state = state.vertical_scroll_state.content_length(num_lines);
    
    let paragraph = Paragraph::new(settings_json_pretty)
        .block(Block::default().borders(Borders::ALL).title(state.setting.id.clone()))
        .scroll((state.vertical_scroll as u16, 0));
    
    frame.render_widget(paragraph, chunks[0]);
    frame.render_stateful_widget(Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("^"))
        .end_symbol(Some("v")),
    chunks[0],
    &mut state.vertical_scroll_state);
    read_write_tables(state, chunks[1], frame)
}


pub struct TableData {
    queue: String,
    
    // Possibly a little chart here maybe?
    
    events_written: u32,
}


impl TableData {
    pub fn new(queue: &str, events: &[QueueStats]) -> Self {
        let mut event_count = 0;
        for (i, event) in events.iter().rev().enumerate() {
            if i <= 2 {
                event_count += event.units
            }
        }
        
        Self{
            queue: queue.replace("queue:", ""),
            events_written: event_count,
        }
    }
    fn ref_array(&self) -> [String; 2] {
        [self.queue.clone(), self.events_written.to_string()]
    }
}

pub fn read_write_tables(state: &BotViewState, area: Rect, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            &[
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
        )
        .split(area);
    
    let header_style = Style::default()
        .fg(tailwind::SLATE.c200)
        .bg(tailwind::BLUE.c900);
    
    if !state.write_stats.is_empty(){
    
        let header  = ["QUEUE", "EVENTS WRITTEN (45 min)"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);
        
        let write_rows = state.write_stats.iter().enumerate()
            .map(|(i, (queue, stats))| {
                let color = match i % 2 {
                            0 => tailwind::SLATE.c950,
                            _ => tailwind::SLATE.c900,
                        };
                let t_data = TableData::new(queue, &stats);
                let item = t_data.ref_array();
                item.into_iter()
                    .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
                    .collect::<Row>()
                    .style(Style::new().fg(tailwind::SLATE.c200).bg(color))
                    .height(2)
            });
        
        let write_table = Table::new(write_rows, [
            Constraint::Length(31),
            Constraint::Length(35)
        ])
        .header(header.clone())
        .column_spacing(2)
        .bg(tailwind::SLATE.c950);
        
        frame.render_widget(write_table, chunks[0])
    }
    if !state.read_stats.is_empty() {
        let header  = ["QUEUE", "EVENTS WRITTEN (45 min)"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);
        
        let read_rows = state.read_stats.iter().enumerate()
            .map(|(i, (queue, stats))| {
                let color = match i % 2 {
                            0 => tailwind::SLATE.c950,
                            _ => tailwind::SLATE.c900,
                        };
                let t_data = TableData::new(queue, &stats);
                let item = t_data.ref_array();
                item.into_iter()
                    .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
                    .collect::<Row>()
                    .style(Style::new().fg(tailwind::SLATE.c200).bg(color))
                    .height(2)
                    
            });
        
        let read_table = Table::new(read_rows, [
            Constraint::Length(31),
            Constraint::Length(35)
        ])
        .header(header.clone())
        .column_spacing(2)
        .bg(tailwind::SLATE.c950);
        
        frame.render_widget(read_table, chunks[1])
    }
    
    
}


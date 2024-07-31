use chrono::{DateTime, Duration, Utc};
use color_eyre::{eyre::bail, owo_colors::OwoColorize};
use ratatui::{layout::Rect, prelude::*, symbols, widgets::{block::Title, Axis, Block, Chart, Dataset, GraphType, LegendPosition}};
use style::Styled;

use crate::app::AppState;




fn calculate_even_distance(min: f64, max: f64, num_divisions: f64) -> f64 {
    (min + max) / num_divisions
}

fn get_even_distance_points(min: f64, max: f64, num_divisions: u32) -> Vec<f64> {
    let distance = calculate_even_distance(min, max, num_divisions as f64);
    let mut points = vec![];
    points.push(min);
    
    let mut prev_point = min;
    while prev_point < max {
        prev_point += distance;
        points.push(prev_point)
    }
    
    points
}

fn convert_points_to_labels(points: &[f64]) -> Vec<Span<'_>> {
    let point_len = points.len();
    points.iter()
        .enumerate()
        .map(|(index, point)| {
            if index == 0 ||index == point_len - 1{
                point.to_string().bold()
            } else {
                point.to_string().into()
            }
        }).collect()
}

fn convert_timestamps_to_human_labels (points: &[f64]) -> Vec<Span<'_>> {
    let point_len = points.len();
    points.iter()
        .enumerate()
        .map(|(index, point)| {
            let date = DateTime::from_timestamp(*point as i64, 0)
                .unwrap()
                .format("%m/%d-%H:%M")
                .to_string();
            if index == 0 ||index == point_len - 1{
                
                date.bold()
                // point.to_string().bold()
            } else {
                date.into()
            }
        }).collect()
}

pub fn render_executions (frame: &mut Frame, area: Rect, app: &AppState) {
    let data = vec![
        (1., 4.), 
        (2., 3.), 
        (3., 1.), 
        (4., 6.)];
    
    let dataset = vec![
        Dataset::default()
            .name("test".italic())
            .marker(symbols::Marker::Dot)
            .style(Style::default().fg(Color::Yellow))
            .graph_type(GraphType::Line)
            .data(&data)
    ];
    
    let x_labels = vec![0., 2., 4.];
    let y_labels = vec![0., 4., 8.];
    
    let chart = Chart::new(dataset)
        .block(Block::bordered().title(
            Title::default()
                .content("Executions".cyan().bold())
                .alignment(Alignment::Center)
            )
        )
        .x_axis(Axis::default()
            .title("Time")
            .style(Style::default().gray())
            .bounds([1., 4.])
            .labels(convert_points_to_labels(&x_labels))
        )
        .y_axis(Axis::default()
            .title("Executions")
            .style(Style::default().gray())
            .bounds([0., 7.])
            .labels(convert_points_to_labels(&y_labels))
        )
        .legend_position(Some(LegendPosition::TopLeft))
        .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)));
    
    frame.render_widget(chart, area)
}

pub fn render_executions_full (frame: &mut Frame, area: Rect, app: &AppState) {
    let data = app.get_chart_data();
    
    let min_x = data.iter().min_by(|(time_a, _), (time_b, _)| time_a.total_cmp(time_b)).unwrap().0;
    let max_x = data.iter().max_by(|(time_a, _), (time_b, _)| time_a.total_cmp(time_b)).unwrap().0;
    
    let min_y = data.iter().min_by(|(_, execution_a), (_, execution_b)| execution_a.total_cmp(execution_b)).unwrap().0;
    let max_y = data.iter().max_by(|(_, execution_a), (_, execution_b)| execution_a.total_cmp(execution_b)).unwrap().0;
    
    let x_label_points = get_even_distance_points(min_x, max_x, 3);
    let y_label_points = get_even_distance_points(min_y, max_y, 3);
    
    let x_labels = convert_timestamps_to_human_labels(&x_label_points);
    if x_labels.is_empty() {
        panic!("x_labels is empty");
    }
    let y_labels = convert_points_to_labels(&y_label_points);
    if y_labels.is_empty() || y_labels.len() == 1 {
        println!("{y_labels:?}");
        panic!("y_labels is empty");
    }
    
    let dataset = vec![Dataset::default()
        .name("executions".italic())
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(Color::Yellow))
        .graph_type(GraphType::Line)
        .data(&data)
    ];
    
    if dataset.is_empty() {
        panic!("dataset is empty")
    }
    
    let chart = Chart::new(dataset)
        .block(Block::bordered().title(
            Title::default()
                .content("Executions".cyan().bold())
                .alignment(Alignment::Center)
            )
        )
        .x_axis(Axis::default()
            .title("Time")
            .style(Style::default().gray())
            .bounds([min_x, max_x])
            .labels(convert_timestamps_to_human_labels(&x_label_points))
        )
        .y_axis(Axis::default()
            .title("Executions")
            .style(Style::default().gray())
            .bounds([min_y, max_y])
            .labels(convert_points_to_labels(&y_label_points))
        )
        .legend_position(Some(LegendPosition::TopLeft))
        .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)));
    
    frame.render_widget(chart, area)
    
    
}

#[cfg(test)]
mod chart_tests {
    use super::{calculate_even_distance, get_even_distance_points};

    
    #[test]
    fn calculate_even_distance_works() {
        let distance = calculate_even_distance(0.0, 10.0, 5.0);
        println!("distance = {distance}");
        assert_eq!(distance, 2.0)
    }
    
    #[test]
    fn get_even_distance_points_works() {
        let points = get_even_distance_points(0.0, 10.0, 5);
        println!("points = {points:?}");
        assert_eq!(points, vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0])
    }
}
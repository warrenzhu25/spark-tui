use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

use crate::models::SparkEventLog;
use std::collections::HashMap;

pub struct EnvironmentTab;

impl EnvironmentTab {
    pub fn draw(
        f: &mut Frame,
        area: ratatui::layout::Rect,
        event_log: &SparkEventLog,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25), // Spark Properties
                Constraint::Percentage(25), // System Properties
                Constraint::Percentage(25), // Hadoop Properties
                Constraint::Percentage(25), // Classpath Entries
            ])
            .split(area);

        // Spark Properties
        Self::draw_properties_table(
            f,
            chunks[0],
            "Spark Properties",
            &event_log.environment.spark_properties,
        );

        // System Properties
        Self::draw_properties_table(
            f,
            chunks[1],
            "System Properties",
            &event_log.environment.system_properties,
        );

        // Hadoop Properties
        Self::draw_properties_table(
            f,
            chunks[2],
            "Hadoop Properties",
            &event_log.environment.hadoop_properties,
        );

        // Classpath Entries
        Self::draw_properties_table(
            f,
            chunks[3],
            "Classpath Entries",
            &event_log.environment.classpath_entries,
        );
    }

    fn draw_properties_table(
        f: &mut Frame,
        area: ratatui::layout::Rect,
        title: &str,
        properties: &HashMap<String, String>,
    ) {
        if properties.is_empty() {
            let paragraph = ratatui::widgets::Paragraph::new("No properties available")
                .block(Block::default().borders(Borders::ALL).title(title))
                .style(Style::default().fg(Color::Gray));
            f.render_widget(paragraph, area);
            return;
        }

        let header_cells = ["Property", "Value"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));

        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let mut properties_vec: Vec<_> = properties.iter().collect();
        properties_vec.sort_by_key(|(key, _)| key.as_str());

        let rows = properties_vec.iter().map(|(key, value)| {
            // Truncate long values for display
            let display_value = if value.len() > 60 {
                format!("{}...", &value[..57])
            } else {
                value.to_string()
            };

            Row::new(vec![
                Cell::from(key.as_str()),
                Cell::from(display_value),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(30), // Property
                Constraint::Percentage(70), // Value
            ]
        )
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(title))
            .column_spacing(1)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        // Create a table state for this specific table (no selection needed for environment)
        let mut table_state = TableState::default();
        f.render_stateful_widget(table, area, &mut table_state);
    }
}
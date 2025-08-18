use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

use crate::models::{StageStatus, SparkEventLog};

pub struct StagesTab;

impl StagesTab {
    pub fn draw(
        f: &mut Frame,
        area: ratatui::layout::Rect,
        event_log: &SparkEventLog,
        table_state: &TableState,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(4), Constraint::Min(0)])
            .split(area);

        // Summary section
        let summary_text = format!(
            "Total Stages: {} | Active: {} | Complete: {} | Failed: {} | Pending: {}",
            event_log.stages.len(),
            event_log.stages.values().filter(|s| matches!(s.status, StageStatus::Active)).count(),
            event_log.stages.values().filter(|s| matches!(s.status, StageStatus::Complete)).count(),
            event_log.stages.values().filter(|s| matches!(s.status, StageStatus::Failed)).count(),
            event_log.stages.values().filter(|s| matches!(s.status, StageStatus::Pending)).count(),
        );

        let summary = ratatui::widgets::Paragraph::new(summary_text)
            .block(Block::default().borders(Borders::ALL).title("Stages Summary"))
            .style(Style::default().fg(Color::White));

        f.render_widget(summary, chunks[0]);

        // Stages table
        let header_cells = ["Stage ID", "Name", "Status", "Tasks", "Submission Time", "Duration", "RDDs"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));

        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let mut stages: Vec<_> = event_log.stages.values().collect();
        stages.sort_by_key(|stage| stage.stage_id);

        let rows = stages.iter().map(|stage| {
            let duration = if let (Some(submission), Some(completion)) = (stage.submission_time, stage.completion_time) {
                format!("{}ms", (completion - submission).num_milliseconds())
            } else if stage.submission_time.is_some() {
                "Running".to_string()
            } else {
                "Pending".to_string()
            };

            let status_style = match stage.status {
                StageStatus::Active => Style::default().fg(Color::Blue),
                StageStatus::Complete => Style::default().fg(Color::Green),
                StageStatus::Failed => Style::default().fg(Color::Red),
                StageStatus::Pending => Style::default().fg(Color::Gray),
            };

            let status_text = match stage.status {
                StageStatus::Active => "ACTIVE",
                StageStatus::Complete => "COMPLETE",
                StageStatus::Failed => "FAILED",
                StageStatus::Pending => "PENDING",
            };

            let submission_time = stage.submission_time
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "N/A".to_string());

            Row::new(vec![
                Cell::from(stage.stage_id.to_string()),
                Cell::from(stage.name.clone()),
                Cell::from(Span::styled(status_text, status_style)),
                Cell::from(stage.num_tasks.to_string()),
                Cell::from(submission_time),
                Cell::from(duration),
                Cell::from(stage.rdd_info.len().to_string()),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(10), // Stage ID
                Constraint::Min(20),    // Name
                Constraint::Length(10), // Status
                Constraint::Length(8),  // Tasks
                Constraint::Length(19), // Submission Time
                Constraint::Length(12), // Duration
                Constraint::Length(6),  // RDDs
            ]
        )
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Stages"))
            .column_spacing(1)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(table, chunks[1], &mut table_state.clone());
    }
}
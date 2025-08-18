use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

use crate::models::{JobStatus, SparkEventLog};

pub struct JobsTab;

impl JobsTab {
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
            "Total Jobs: {} | Active: {} | Completed: {} | Failed: {}",
            event_log.jobs.len(),
            event_log.jobs.values().filter(|j| matches!(j.status, JobStatus::Running)).count(),
            event_log.jobs.values().filter(|j| matches!(j.status, JobStatus::Succeeded)).count(),
            event_log.jobs.values().filter(|j| matches!(j.status, JobStatus::Failed)).count(),
        );

        let summary = ratatui::widgets::Paragraph::new(summary_text)
            .block(Block::default().borders(Borders::ALL).title("Jobs Summary"))
            .style(Style::default().fg(Color::White));

        f.render_widget(summary, chunks[0]);

        // Jobs table
        let header_cells = ["Job ID", "Description", "Status", "Submission Time", "Duration", "Stages", "Tasks"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));

        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let mut jobs: Vec<_> = event_log.jobs.values().collect();
        jobs.sort_by_key(|job| job.job_id);

        let rows = jobs.iter().map(|job| {
            let duration = if let Some(completion_time) = job.completion_time {
                format!("{}ms", (completion_time - job.submission_time).num_milliseconds())
            } else {
                "Running".to_string()
            };

            let status_style = match job.status {
                JobStatus::Running => Style::default().fg(Color::Blue),
                JobStatus::Succeeded => Style::default().fg(Color::Green),
                JobStatus::Failed => Style::default().fg(Color::Red),
                JobStatus::Unknown => Style::default().fg(Color::Gray),
            };

            let status_text = match job.status {
                JobStatus::Running => "RUNNING",
                JobStatus::Succeeded => "SUCCEEDED",
                JobStatus::Failed => "FAILED",
                JobStatus::Unknown => "UNKNOWN",
            };

            Row::new(vec![
                Cell::from(job.job_id.to_string()),
                Cell::from(job.description.as_deref().unwrap_or(&job.name)),
                Cell::from(Span::styled(status_text, status_style)),
                Cell::from(job.submission_time.format("%Y-%m-%d %H:%M:%S").to_string()),
                Cell::from(duration),
                Cell::from(job.stage_ids.len().to_string()),
                Cell::from(format!("{}/{}", job.num_completed_tasks, job.num_tasks)),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(8),  // Job ID
                Constraint::Min(20),    // Description
                Constraint::Length(10), // Status
                Constraint::Length(19), // Submission Time
                Constraint::Length(12), // Duration
                Constraint::Length(8),  // Stages
                Constraint::Length(12), // Tasks
            ]
        )
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Jobs"))
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
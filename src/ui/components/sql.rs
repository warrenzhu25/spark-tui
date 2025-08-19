use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table, TableState, Paragraph, Wrap},
    Frame,
};

use crate::models::{SqlExecutionStatus, SparkEventLog};

pub struct SqlTab;

impl SqlTab {
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
        let total_executions = event_log.sql_executions.len();
        let completed_executions = event_log.sql_executions.values()
            .filter(|e| matches!(e.status, SqlExecutionStatus::Completed))
            .count();
        let failed_executions = event_log.sql_executions.values()
            .filter(|e| matches!(e.status, SqlExecutionStatus::Failed))
            .count();
        let running_executions = event_log.sql_executions.values()
            .filter(|e| matches!(e.status, SqlExecutionStatus::Running))
            .count();

        let summary_text = if total_executions > 0 {
            format!(
                "SQL Executions: {} | Completed: {} | Failed: {} | Running: {}",
                total_executions,
                completed_executions,
                failed_executions,
                running_executions
            )
        } else {
            "No SQL executions found in this application".to_string()
        };

        let summary = Paragraph::new(summary_text)
            .block(Block::default().borders(Borders::ALL).title("SQL Summary"))
            .style(Style::default().fg(Color::White));

        f.render_widget(summary, chunks[0]);

        if total_executions == 0 {
            // Show a message when no SQL executions are found
            let no_sql_message = Paragraph::new(
                "This application did not execute any SQL queries.\n\n\
                The SQL tab shows information about Spark SQL executions including:\n\
                • Query descriptions and execution plans\n\
                • Execution timing and status\n\
                • Associated jobs and stages\n\
                • SQL metrics and performance data"
            )
                .block(Block::default().borders(Borders::ALL).title("SQL Executions"))
                .style(Style::default().fg(Color::Gray))
                .wrap(Wrap { trim: true });

            f.render_widget(no_sql_message, chunks[1]);
            return;
        }

        // SQL executions table
        let header_cells = [
            "Execution ID", "Description", "Status", "Submission Time", 
            "Duration", "Jobs", "Stages", "Details"
        ]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));

        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let mut executions: Vec<_> = event_log.sql_executions.values().collect();
        executions.sort_by_key(|execution| execution.execution_id);

        let rows = executions.iter().map(|execution| {
            let duration = if let Some(completion_time) = execution.completion_time {
                format!("{}ms", (completion_time - execution.submission_time).num_milliseconds())
            } else {
                "Running".to_string()
            };

            let status_style = match execution.status {
                SqlExecutionStatus::Running => Style::default().fg(Color::Blue),
                SqlExecutionStatus::Completed => Style::default().fg(Color::Green),
                SqlExecutionStatus::Failed => Style::default().fg(Color::Red),
            };

            let status_text = match execution.status {
                SqlExecutionStatus::Running => "RUNNING",
                SqlExecutionStatus::Completed => "COMPLETED",
                SqlExecutionStatus::Failed => "FAILED",
            };

            // Truncate description and details for display
            let display_description = if execution.description.len() > 40 {
                format!("{}...", &execution.description[..37])
            } else {
                execution.description.clone()
            };

            let display_details = if execution.details.len() > 30 {
                format!("{}...", &execution.details[..27])
            } else if execution.details.is_empty() {
                "N/A".to_string()
            } else {
                execution.details.clone()
            };

            Row::new(vec![
                Cell::from(execution.execution_id.to_string()),
                Cell::from(display_description),
                Cell::from(Span::styled(status_text, status_style)),
                Cell::from(execution.submission_time.format("%H:%M:%S").to_string()),
                Cell::from(duration),
                Cell::from(execution.jobs.len().to_string()),
                Cell::from(execution.stages.len().to_string()),
                Cell::from(display_details),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(12), // Execution ID
                Constraint::Min(30),    // Description
                Constraint::Length(10), // Status
                Constraint::Length(12), // Submission Time
                Constraint::Length(10), // Duration
                Constraint::Length(6),  // Jobs
                Constraint::Length(7),  // Stages
                Constraint::Min(20),    // Details
            ]
        )
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("SQL Executions"))
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
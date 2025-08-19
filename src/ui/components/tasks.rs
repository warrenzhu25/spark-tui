use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

use crate::models::{TaskStatus, SparkEventLog};

pub struct TasksTab;

impl TasksTab {
    pub fn draw(
        f: &mut Frame,
        area: ratatui::layout::Rect,
        event_log: &SparkEventLog,
        table_state: &TableState,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(6), Constraint::Min(0)])
            .split(area);

        // Summary section
        let total_tasks = event_log.tasks.len();
        let successful_tasks = event_log.tasks.values().filter(|t| matches!(t.status, TaskStatus::Success)).count();
        let failed_tasks = event_log.tasks.values().filter(|t| matches!(t.status, TaskStatus::Failed)).count();
        let killed_tasks = event_log.tasks.values().filter(|t| matches!(t.status, TaskStatus::Killed)).count();
        let running_tasks = event_log.tasks.values().filter(|t| matches!(t.status, TaskStatus::Running)).count();

        // Calculate total execution time and data processed
        let total_execution_time: u64 = event_log.tasks.values()
            .filter_map(|t| t.metrics.as_ref())
            .map(|m| m.execution_time)
            .sum();

        let total_input_bytes: u64 = event_log.tasks.values()
            .filter_map(|t| t.metrics.as_ref())
            .filter_map(|m| m.input_metrics.as_ref())
            .map(|i| i.bytes_read)
            .sum();

        let summary_text = format!(
            "Total Tasks: {} | Success: {} | Failed: {} | Killed: {} | Running: {}\nTotal Execution Time: {:.1}s | Total Input Data: {}",
            total_tasks,
            successful_tasks,
            failed_tasks, 
            killed_tasks,
            running_tasks,
            total_execution_time as f64 / 1000.0,
            format_bytes(total_input_bytes)
        );

        let summary = ratatui::widgets::Paragraph::new(summary_text)
            .block(Block::default().borders(Borders::ALL).title("Tasks Summary"))
            .style(Style::default().fg(Color::White));

        f.render_widget(summary, chunks[0]);

        // Tasks table
        let header_cells = [
            "Task ID", "Stage", "Partition", "Status", "Executor", "Host", 
            "Launch Time", "Duration", "CPU Time", "GC Time", "Input", "Output", "Spilled"
        ]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));

        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let mut tasks: Vec<_> = event_log.tasks.values().collect();
        tasks.sort_by_key(|task| task.task_id);

        let rows = tasks.iter().map(|task| {
            let duration = if let Some(finish_time) = task.finish_time {
                format!("{}ms", (finish_time - task.launch_time).num_milliseconds())
            } else {
                "Running".to_string()
            };

            let status_style = match task.status {
                TaskStatus::Running => Style::default().fg(Color::Blue),
                TaskStatus::Success => Style::default().fg(Color::Green),
                TaskStatus::Failed => Style::default().fg(Color::Red),
                TaskStatus::Killed => Style::default().fg(Color::Magenta),
            };

            let status_text = match task.status {
                TaskStatus::Running => "RUNNING",
                TaskStatus::Success => "SUCCESS",
                TaskStatus::Failed => "FAILED",
                TaskStatus::Killed => "KILLED",
            };

            // Extract metrics
            let (cpu_time, gc_time, input_data, output_data, spilled_data) = if let Some(metrics) = &task.metrics {
                (
                    format!("{}ms", metrics.cpu_time),
                    format!("{}ms", metrics.gc_time),
                    metrics.input_metrics.as_ref()
                        .map(|i| format_bytes(i.bytes_read))
                        .unwrap_or_else(|| "0 B".to_string()),
                    metrics.output_metrics.as_ref()
                        .map(|o| format_bytes(o.bytes_written))
                        .unwrap_or_else(|| "0 B".to_string()),
                    format_bytes(metrics.memory_bytes_spilled + metrics.disk_bytes_spilled),
                )
            } else {
                ("N/A".to_string(), "N/A".to_string(), "N/A".to_string(), "N/A".to_string(), "N/A".to_string())
            };

            Row::new(vec![
                Cell::from(task.task_id.to_string()),
                Cell::from(format!("{}.{}", task.stage_id, task.stage_attempt_id)),
                Cell::from(task.partition_id.to_string()),
                Cell::from(Span::styled(status_text, status_style)),
                Cell::from(task.executor_id.clone()),
                Cell::from(task.host.clone()),
                Cell::from(task.launch_time.format("%H:%M:%S").to_string()),
                Cell::from(duration),
                Cell::from(cpu_time),
                Cell::from(gc_time),
                Cell::from(input_data),
                Cell::from(output_data),
                Cell::from(spilled_data),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(8),  // Task ID
                Constraint::Length(8),  // Stage
                Constraint::Length(9),  // Partition
                Constraint::Length(8),  // Status
                Constraint::Length(10), // Executor
                Constraint::Length(15), // Host
                Constraint::Length(10), // Launch Time
                Constraint::Length(10), // Duration
                Constraint::Length(8),  // CPU Time
                Constraint::Length(8),  // GC Time
                Constraint::Length(10), // Input
                Constraint::Length(10), // Output
                Constraint::Length(10), // Spilled
            ]
        )
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Tasks"))
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

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
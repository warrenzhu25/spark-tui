use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

use crate::models::SparkEventLog;

pub struct ExecutorsTab;

impl ExecutorsTab {
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
        let active_executors = event_log.executors.values().filter(|e| e.is_active).count();
        let total_cores: u32 = event_log.executors.values().map(|e| e.total_cores).sum();
        let total_memory: u64 = event_log.executors.values().map(|e| e.max_memory).sum();
        let memory_used: u64 = event_log.executors.values().map(|e| e.memory_used).sum();

        let summary_text = format!(
            "Total Executors: {} | Active: {} | Total Cores: {} | Total Memory: {} MB | Used Memory: {} MB",
            event_log.executors.len(),
            active_executors,
            total_cores,
            total_memory / (1024 * 1024),
            memory_used / (1024 * 1024),
        );

        let tasks_summary = format!(
            "Total Tasks: {} | Completed: {} | Failed: {}",
            event_log.executors.values().map(|e| e.total_tasks).sum::<u32>(),
            event_log.executors.values().map(|e| e.completed_tasks).sum::<u32>(),
            event_log.executors.values().map(|e| e.failed_tasks).sum::<u32>(),
        );

        let summary = ratatui::widgets::Paragraph::new(format!("{}\n{}", summary_text, tasks_summary))
            .block(Block::default().borders(Borders::ALL).title("Executors Summary"))
            .style(Style::default().fg(Color::White));

        f.render_widget(summary, chunks[0]);

        // Executors table
        let header_cells = ["Executor ID", "Host", "Status", "Cores", "Memory", "Tasks", "Failed", "GC Time", "Input", "Shuffle Read", "Shuffle Write"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));

        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let mut executors: Vec<_> = event_log.executors.values().collect();
        executors.sort_by(|a, b| {
            // Sort by executor ID, treating numeric parts as numbers
            match (a.executor_id.parse::<i32>(), b.executor_id.parse::<i32>()) {
                (Ok(a_num), Ok(b_num)) => a_num.cmp(&b_num),
                _ => a.executor_id.cmp(&b.executor_id),
            }
        });

        let rows = executors.iter().map(|executor| {
            let status_style = if executor.is_active {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            let status_text = if executor.is_active { "ACTIVE" } else { "REMOVED" };

            let memory_usage = if executor.max_memory > 0 {
                format!(
                    "{:.1}% ({}/{})",
                    (executor.memory_used as f64 / executor.max_memory as f64) * 100.0,
                    format_bytes(executor.memory_used),
                    format_bytes(executor.max_memory)
                )
            } else {
                "N/A".to_string()
            };

            let gc_time_sec = executor.total_gc_time as f64 / 1000.0;

            Row::new(vec![
                Cell::from(executor.executor_id.clone()),
                Cell::from(executor.host.clone()),
                Cell::from(Span::styled(status_text, status_style)),
                Cell::from(executor.total_cores.to_string()),
                Cell::from(memory_usage),
                Cell::from(format!("{}/{}", executor.completed_tasks, executor.total_tasks)),
                Cell::from(executor.failed_tasks.to_string()),
                Cell::from(format!("{:.1}s", gc_time_sec)),
                Cell::from(format_bytes(executor.total_input_bytes)),
                Cell::from(format_bytes(executor.total_shuffle_read)),
                Cell::from(format_bytes(executor.total_shuffle_write)),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(12), // Executor ID
                Constraint::Length(15), // Host
                Constraint::Length(8),  // Status
                Constraint::Length(6),  // Cores
                Constraint::Length(18), // Memory
                Constraint::Length(10), // Tasks
                Constraint::Length(8),  // Failed
                Constraint::Length(8),  // GC Time
                Constraint::Length(10), // Input
                Constraint::Length(12), // Shuffle Read
                Constraint::Length(12), // Shuffle Write
            ]
        )
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Executors"))
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
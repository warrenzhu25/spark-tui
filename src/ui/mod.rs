pub mod components;
pub mod state;

pub use components::*;
pub use state::*;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::models::SparkEventLog;

pub struct UI;

impl UI {
    pub fn new() -> Self {
        Self
    }

    pub fn draw(&self, f: &mut Frame, event_log: &SparkEventLog, state: &AppState) {
        let size = f.area();

        // Create the main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header with app info
                Constraint::Length(3), // Tab bar
                Constraint::Min(0),    // Content area
                Constraint::Length(3), // Footer with help
            ])
            .split(size);

        // Draw header
        self.draw_header(f, chunks[0], event_log);

        // Draw tab bar
        self.draw_tab_bar(f, chunks[1], state);

        // Draw content based on selected tab
        match state.selected_tab {
            TabIndex::Jobs => {
                JobsTab::draw(f, chunks[2], event_log, &state.jobs_table_state);
            }
            TabIndex::Stages => {
                StagesTab::draw(f, chunks[2], event_log, &state.stages_table_state);
            }
            TabIndex::Tasks => {
                TasksTab::draw(f, chunks[2], event_log, &state.tasks_table_state);
            }
            TabIndex::Executors => {
                ExecutorsTab::draw(f, chunks[2], event_log, &state.executors_table_state);
            }
            TabIndex::Environment => {
                EnvironmentTab::draw(f, chunks[2], event_log);
            }
        }

        // Draw footer
        self.draw_footer(f, chunks[3]);
    }

    fn draw_header(&self, f: &mut Frame, area: ratatui::layout::Rect, event_log: &SparkEventLog) {
        let app_info = &event_log.application_info;
        let duration = if let Some(end_time) = app_info.end_time {
            format!(" ({}ms)", (end_time - app_info.start_time).num_milliseconds())
        } else {
            " (Running)".to_string()
        };

        let header_text = format!(
            "Application: {} | ID: {} | User: {} | Spark Version: {}{}",
            app_info.app_name,
            app_info.app_id,
            app_info.user,
            app_info.spark_version,
            duration
        );

        let paragraph = Paragraph::new(header_text)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL).title("Spark Application"));

        f.render_widget(paragraph, area);
    }

    fn draw_tab_bar(&self, f: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
        let tab_titles = vec!["Jobs (1)", "Stages (2)", "Tasks (3)", "Executors (4)", "Environment (5)"];
        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Blue)
                    .fg(Color::White),
            )
            .select(state.selected_tab as usize);

        f.render_widget(tabs, area);
    }

    fn draw_footer(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let help_text = "Navigation: Tab/Shift+Tab (Switch tabs) | 1-5 (Direct tab) | ↑↓/jk (Navigate) | q/Esc (Quit)";
        let paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("Help"));

        f.render_widget(paragraph, area);
    }
}
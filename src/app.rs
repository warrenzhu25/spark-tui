use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::time::Duration;

use crate::events::{AppEvent, EventHandler};
use crate::models::SparkEventLog;
use crate::ui::{AppState, TabIndex, UI};

pub struct App {
    event_log: SparkEventLog,
    should_quit: bool,
    ui: UI,
    state: AppState,
}

impl App {
    pub fn new(event_log: SparkEventLog) -> Self {
        Self {
            event_log,
            should_quit: false,
            ui: UI::new(),
            state: AppState::new(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Event handler
        let mut event_handler = EventHandler::new(Duration::from_millis(100));

        // Main loop
        while !self.should_quit {
            // Draw UI
            terminal.draw(|f| {
                self.ui.draw(f, &self.event_log, &self.state);
            })?;

            // Handle events
            match event_handler.next()? {
                AppEvent::Key(key) => self.handle_key_event(key),
                AppEvent::Quit => self.should_quit = true,
                AppEvent::Tick => {
                    // Handle periodic updates if needed
                }
            }
        }

        // Cleanup terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char('1') => {
                self.state.selected_tab = TabIndex::Jobs;
            }
            KeyCode::Char('2') => {
                self.state.selected_tab = TabIndex::Stages;
            }
            KeyCode::Char('3') => {
                self.state.selected_tab = TabIndex::Tasks;
            }
            KeyCode::Char('4') => {
                self.state.selected_tab = TabIndex::Executors;
            }
            KeyCode::Char('5') => {
                self.state.selected_tab = TabIndex::Environment;
            }
            KeyCode::Tab => {
                self.state.next_tab();
            }
            KeyCode::BackTab => {
                self.state.previous_tab();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                match self.state.selected_tab {
                    TabIndex::Jobs => {
                        if self.state.jobs_table_state.selected().unwrap_or(0) > 0 {
                            let selected = self.state.jobs_table_state.selected().unwrap_or(0);
                            self.state.jobs_table_state.select(Some(selected - 1));
                        }
                    }
                    TabIndex::Stages => {
                        if self.state.stages_table_state.selected().unwrap_or(0) > 0 {
                            let selected = self.state.stages_table_state.selected().unwrap_or(0);
                            self.state.stages_table_state.select(Some(selected - 1));
                        }
                    }
                    TabIndex::Tasks => {
                        if self.state.tasks_table_state.selected().unwrap_or(0) > 0 {
                            let selected = self.state.tasks_table_state.selected().unwrap_or(0);
                            self.state.tasks_table_state.select(Some(selected - 1));
                        }
                    }
                    TabIndex::Executors => {
                        if self.state.executors_table_state.selected().unwrap_or(0) > 0 {
                            let selected = self.state.executors_table_state.selected().unwrap_or(0);
                            self.state.executors_table_state.select(Some(selected - 1));
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                match self.state.selected_tab {
                    TabIndex::Jobs => {
                        let jobs_count = self.event_log.jobs.len();
                        if jobs_count > 0 {
                            let selected = self.state.jobs_table_state.selected().unwrap_or(0);
                            if selected < jobs_count - 1 {
                                self.state.jobs_table_state.select(Some(selected + 1));
                            }
                        }
                    }
                    TabIndex::Stages => {
                        let stages_count = self.event_log.stages.len();
                        if stages_count > 0 {
                            let selected = self.state.stages_table_state.selected().unwrap_or(0);
                            if selected < stages_count - 1 {
                                self.state.stages_table_state.select(Some(selected + 1));
                            }
                        }
                    }
                    TabIndex::Tasks => {
                        let tasks_count = self.event_log.tasks.len();
                        if tasks_count > 0 {
                            let selected = self.state.tasks_table_state.selected().unwrap_or(0);
                            if selected < tasks_count - 1 {
                                self.state.tasks_table_state.select(Some(selected + 1));
                            }
                        }
                    }
                    TabIndex::Executors => {
                        let executors_count = self.event_log.executors.len();
                        if executors_count > 0 {
                            let selected = self.state.executors_table_state.selected().unwrap_or(0);
                            if selected < executors_count - 1 {
                                self.state.executors_table_state.select(Some(selected + 1));
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
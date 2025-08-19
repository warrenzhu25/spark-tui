use ratatui::widgets::TableState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabIndex {
    Jobs = 0,
    Stages = 1,
    Tasks = 2,
    Executors = 3,
    Sql = 4,
    Environment = 5,
}

impl TabIndex {
    pub fn next(&self) -> Self {
        match self {
            TabIndex::Jobs => TabIndex::Stages,
            TabIndex::Stages => TabIndex::Tasks,
            TabIndex::Tasks => TabIndex::Executors,
            TabIndex::Executors => TabIndex::Sql,
            TabIndex::Sql => TabIndex::Environment,
            TabIndex::Environment => TabIndex::Jobs,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            TabIndex::Jobs => TabIndex::Environment,
            TabIndex::Stages => TabIndex::Jobs,
            TabIndex::Tasks => TabIndex::Stages,
            TabIndex::Executors => TabIndex::Tasks,
            TabIndex::Sql => TabIndex::Executors,
            TabIndex::Environment => TabIndex::Sql,
        }
    }
}

pub struct AppState {
    pub selected_tab: TabIndex,
    pub jobs_table_state: TableState,
    pub stages_table_state: TableState,
    pub tasks_table_state: TableState,
    pub executors_table_state: TableState,
    pub sql_table_state: TableState,
}

impl AppState {
    pub fn new() -> Self {
        let mut jobs_table_state = TableState::default();
        jobs_table_state.select(Some(0));

        let mut stages_table_state = TableState::default();
        stages_table_state.select(Some(0));

        let mut tasks_table_state = TableState::default();
        tasks_table_state.select(Some(0));

        let mut executors_table_state = TableState::default();
        executors_table_state.select(Some(0));

        let mut sql_table_state = TableState::default();
        sql_table_state.select(Some(0));

        Self {
            selected_tab: TabIndex::Jobs,
            jobs_table_state,
            stages_table_state,
            tasks_table_state,
            executors_table_state,
            sql_table_state,
        }
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = self.selected_tab.next();
    }

    pub fn previous_tab(&mut self) {
        self.selected_tab = self.selected_tab.previous();
    }
}
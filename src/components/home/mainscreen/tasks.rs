use std::collections::HashMap;

use ratatui::{
    layout::{Constraint, Flex, Layout},
    prelude::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Clear, Row, Table},
    Frame,
};

use crate::{components::traits::renderable::Renderable, queryworker::query::QueryStatus};

#[derive(Clone)]
enum Status {
    Running,
    Failed,
}

impl ToString for Status {
    fn to_string(&self) -> String {
        match self {
            Status::Running => "Running",
            Status::Failed => "Failed!",
        }
        .to_string()
    }
}

pub struct Tasks {
    border: Block<'static>,
    table: Table<'static>,
    tasks: HashMap<usize, (String, Status)>,
    show_internal: bool,
}

impl Tasks {
    pub fn new(show_internal: bool) -> Self {
        Self {
            border: Self::gen_block(),
            table: Table::new(
                [Row::new(vec!["ID", "Task", "Status"])],
                [Constraint::Max(4), Constraint::Min(1), Constraint::Max(10)],
            ),
            tasks: HashMap::new(),
            show_internal,
        }
    }

    fn gen_block() -> Block<'static> {
        let style = Style::new().white();
        let title = Span::styled("Tasks", Style::default().add_modifier(Modifier::BOLD));
        Block::bordered().title(title).border_style(style)
    }

    fn gen_rows(&self) -> Vec<Row<'static>> {
        let mut rows: Vec<Row<'static>> = self
            .tasks
            .clone()
            .into_iter()
            .map(|(id, (msg, status))| Row::new(vec![id.to_string(), msg, status.to_string()]))
            .collect();
        rows.insert(0, Row::new(vec!["ID", "Task", "Status"]));
        rows
    }

    pub fn update_task(&mut self, ticket: &usize, status: &QueryStatus) {
        match status {
            QueryStatus::Finished(_) => {
                self.tasks.remove(ticket);
            }
            QueryStatus::Aborted(cancelled) => {
                if let Some((_, status)) = self.tasks.get_mut(ticket) {
                    if *cancelled {
                        self.tasks.remove(ticket);
                    } else {
                        *status = Status::Failed;
                    }
                }
            }
            QueryStatus::Requested(q) => {
                if q.show_task() {
                    self.tasks.insert(*ticket, (q.to_string(), Status::Running));
                }
            }
        };
        self.table = Table::new(
            self.gen_rows(),
            [Constraint::Max(4), Constraint::Min(1), Constraint::Max(10)],
        );
    }

    pub fn get_task_count(&self) -> usize {
        self.tasks.len()
    }
}

impl Renderable for Tasks {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let vertical = Layout::vertical([Constraint::Percentage(80)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(80)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        frame.render_widget(Clear, area);
        frame.render_widget(&self.border, area);
        frame.render_widget(&self.table, self.border.inner(area));
    }
}

use std::collections::HashMap;

use ratatui::{
    layout::{Constraint, Flex, Layout},
    prelude::Rect,
    style::{Modifier, Style, Stylize},
    text::Span,
    widgets::{Block, Clear, Row, Table},
    Frame,
};

use crate::{
    components::traits::renderable::Renderable,
    queryworker::query::{FromQueryWorker, ToQueryWorker},
};

pub struct Tasks {
    border: Block<'static>,
    table: Table<'static>,
    tasks: HashMap<usize, ToQueryWorker>,
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
            .map(|(id, t)| {
                Row::new(vec![
                    id.to_string(),
                    t.query.to_string(),
                    "Running".to_string(),
                ])
            })
            .collect();
        rows.insert(0, Row::new(vec!["ID", "Task", "Status"]));
        rows
    }

    pub fn register_task(&mut self, task: ToQueryWorker) {
        if !task.query.has_reply() {
            return;
        }
        if task.query.is_internal() && !self.show_internal {
            return;
        }
        self.tasks.insert(task.ticket, task);
        self.table = Table::new(
            self.gen_rows(),
            [Constraint::Max(4), Constraint::Min(1), Constraint::Max(10)],
        );
    }

    pub fn unregister_task(&mut self, task: &FromQueryWorker) {
        self.tasks.remove(&task.ticket);
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

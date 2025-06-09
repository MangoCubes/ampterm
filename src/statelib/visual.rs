use ratatui::{
    layout::Constraint,
    text::Text,
    widgets::{Cell, List, ListItem, ListState, Row, Table, TableState},
};

struct Range {
    start: usize,
    end: usize,
}

enum VisualMode {
    // Visual mode disabled
    Off,
    // Visual mode enabled
    Select(usize),
    // Visual mode enabled (deselect selected region)
    Deselect(usize),
}

pub struct Visual<'a, T> {
    items: Vec<T>,
    temp: VisualMode,
    // List of all selected items
    selected: Vec<Range>,
    tablestate: TableState,
    to_row: fn(&T) -> Row<'a>,
    widths: Vec<Constraint>,
}

impl<'a, T> Visual<'a, T> {
    pub fn new(list: Vec<T>, func: fn(&T) -> Row<'a>, widths: Vec<Constraint>) -> Self {
        let mut tablestate = TableState::default();
        tablestate.select(Some(0));
        Self {
            items: list,
            temp: VisualMode::Off,
            selected: Vec::new(),
            tablestate,
            to_row: func,
            widths,
        }
    }
    fn gen_list(&self, current: usize) -> Table {
        let rows: Vec<Row> = self.items.iter().map(|item| (self.to_row)(item)).collect();
        Table::new(rows, &self.widths)
    }
    pub fn enable_visual(&mut self, deselect: bool) -> Table {
        let Some(current) = self.tablestate.selected() else {
            panic!("Invalid cursor location!");
        };
        self.temp = if deselect {
            VisualMode::Deselect(current)
        } else {
            VisualMode::Select(current)
        };
        self.gen_list(current)
    }
}

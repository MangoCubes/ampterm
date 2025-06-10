use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Row, Table, TableState},
    Frame,
};

struct Range {
    start: usize,
    end: usize,
}

enum VisualMode {
    // Visual mode disabled
    Off,
    // Visual mode enabled
    // The number represent the start of the region
    Select(usize),
    // Visual mode enabled (deselect selected region)
    // The number represent the start of the region
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
    comp: Table<'a>,
}

impl<'a, T> Visual<'a, T> {
    fn gen_table(
        items: &Vec<T>,
        to_row: &fn(&T) -> Row<'a>,
        widths: &Vec<Constraint>,
    ) -> Table<'a> {
        let rows: Vec<Row> = items.iter().map(|item| (to_row)(item)).collect();
        Table::new(rows, widths)
    }
    pub fn new(list: Vec<T>, func: fn(&T) -> Row<'a>, widths: Vec<Constraint>) -> Self {
        let mut tablestate = TableState::default();
        tablestate.select(Some(0));
        let comp = Self::gen_table(&list, &func, &widths);
        Self {
            items: list,
            temp: VisualMode::Off,
            selected: Vec::new(),
            tablestate,
            to_row: func,
            widths,
            comp,
        }
    }
    pub fn enable_visual(&mut self, deselect: bool) {
        let Some(current) = self.tablestate.selected() else {
            panic!("Invalid cursor location!");
        };
        self.temp = if deselect {
            VisualMode::Deselect(current)
        } else {
            VisualMode::Select(current)
        };
        self.comp = Self::gen_table(&self.items, &self.to_row, &self.widths);
    }
    #[inline]
    pub fn get_current(&self) -> &T {
        let Some(current) = self.tablestate.selected() else {
            panic!("Invalid cursor location!");
        };
        &self.items[current]
    }
    #[inline]
    pub fn select_first(&mut self) {
        self.tablestate.select_first();
    }
    #[inline]
    pub fn select_last(&mut self) {
        self.tablestate.select_last();
    }
    #[inline]
    pub fn select_next(&mut self) {
        self.tablestate.select_next();
    }
    #[inline]
    pub fn select_previous(&mut self) {
        self.tablestate.select_previous();
    }
    pub fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_stateful_widget(&self.comp, area, &mut self.tablestate);
        Ok(())
    }

    pub fn disable_visual(&mut self, apply: bool) {
        if apply {
            todo!();
        };
        self.temp = VisualMode::Off;
        self.comp = Self::gen_table(&self.items, &self.to_row, &self.widths);
    }
}

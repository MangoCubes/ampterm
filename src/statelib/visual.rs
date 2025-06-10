use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Row, Table, TableState},
    Frame,
};

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
    selected: Vec<bool>,
    tablestate: TableState,
    to_row: fn(&T) -> Row<'a>,
    widths: Vec<Constraint>,
    comp: Table<'a>,
}

impl<'a, T> Visual<'a, T> {
    fn gen_table(&self) -> Table<'a> {
        let iter = self.items.iter().enumerate();
        let rows: Vec<Row> =
            if let VisualMode::Select(start) | VisualMode::Deselect(start) = self.temp {
                let end = self.tablestate.selected().unwrap();
                let (a, b) = if start < end {
                    (start, end)
                } else {
                    (end, start)
                };
                iter.map(|(i, item)| {
                    let mut row = (self.to_row)(item);
                    row = if i <= b && i >= a {
                        row.reversed()
                    } else {
                        row
                    };
                    if self.selected[i] {
                        row.green()
                    } else {
                        row
                    }
                })
                .collect()
            } else {
                iter.map(|(i, item)| {
                    let row = (self.to_row)(item);
                    if self.selected[i] {
                        row.reversed()
                    } else {
                        row
                    }
                })
                .collect()
            };
        Table::new(rows, &self.widths)
            .highlight_symbol(">")
            .row_highlight_style(Style::new().reversed())
    }
    pub fn new(list: Vec<T>, to_row: fn(&T) -> Row<'a>, widths: Vec<Constraint>) -> Self {
        let mut tablestate = TableState::default();
        let len = list.len();
        tablestate.select(Some(0));
        let rows: Vec<Row> = list.iter().map(|item| (to_row)(item)).collect();
        let comp = Table::new(rows, &widths)
            .highlight_symbol(">")
            .row_highlight_style(Style::new().reversed());
        Self {
            items: list,
            temp: VisualMode::Off,
            selected: vec![false; len],
            tablestate,
            to_row,
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
        self.comp = self.gen_table();
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
        self.comp = self.gen_table();
    }
    #[inline]
    pub fn select_last(&mut self) {
        self.tablestate.select_last();
        self.comp = self.gen_table();
    }
    #[inline]
    pub fn select_next(&mut self) {
        self.tablestate.select_next();
        self.comp = self.gen_table();
    }
    #[inline]
    pub fn select_previous(&mut self) {
        self.tablestate.select_previous();
        self.comp = self.gen_table();
    }
    pub fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_stateful_widget(&self.comp, area, &mut self.tablestate);
        Ok(())
    }

    pub fn disable_visual(&mut self, apply: bool) {
        if apply {
            let end = self.tablestate.selected().unwrap();
            if let VisualMode::Select(start) = self.temp {
                for i in start..=end {
                    self.selected[i] = true;
                }
            } else if let VisualMode::Deselect(start) = self.temp {
                for i in start..=end {
                    self.selected[i] = false;
                }
            }
        };
        self.temp = VisualMode::Off;
        self.comp = self.gen_table();
    }
}

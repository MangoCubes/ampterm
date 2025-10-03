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

/// Wrapper for Ratatui's [`Table`]. Comes with the ability to use visual mode seen in Vim.
pub struct VisualTable<'a, T> {
    /// List of all items
    items: Vec<T>,
    /// Current mode of the table
    mode: VisualMode,
    /// List of all selected items
    selected: Vec<bool>,
    /// [`TableState`] is used by Ratatui itself
    tablestate: TableState,
    /// This is a function given to this component at creation to tell [`VisualTable`] how each row
    /// should be made
    to_row: fn(&Vec<T>, &Option<(usize, usize, bool)>, &Vec<bool>) -> Vec<Row<'a>>,
    /// Specifies the width of the table
    widths: Vec<Constraint>,
    /// Actual component used to display the table
    comp: Table<'a>,
}

impl<'a, T> VisualTable<'a, T> {
    fn gen_table(&self) -> Table<'a> {
        let rows: Vec<Row> = (self.to_row)(&self.items, &self.get_range(), &self.selected);
        Table::new(rows, &self.widths)
            .highlight_symbol(">")
            .row_highlight_style(Style::new().reversed())
    }
    pub fn new(
        list: Vec<T>,
        to_row: fn(&Vec<T>, &Option<(usize, usize, bool)>, &Vec<bool>) -> Vec<Row<'a>>,
        widths: Vec<Constraint>,
    ) -> Self {
        let mut tablestate = TableState::default();
        let len = list.len();
        tablestate.select(Some(0));
        let selected = vec![false; len];
        let rows: Vec<Row> = to_row(&list, &None, &selected);
        let comp = Table::new(rows, &widths)
            .highlight_symbol(">")
            .row_highlight_style(Style::new().reversed());
        Self {
            items: list,
            mode: VisualMode::Off,
            selected,
            tablestate,
            to_row,
            widths,
            comp,
        }
    }
    /// Enters visual mode
    /// If [`deselect`] is true, then [`VisualMode::Deselect`] is used instead
    pub fn enable_visual(&mut self, deselect: bool) {
        let Some(current) = self.tablestate.selected() else {
            panic!("Invalid cursor location!");
        };
        self.mode = if deselect {
            VisualMode::Deselect(current)
        } else {
            VisualMode::Select(current)
        };
        self.comp = self.gen_table();
    }
    /// Get temporary selection
    /// Returns index of the start of selection and end, and boolean that indicates selection mode
    /// If true, then the current selection is [`VisualMode::Select`], false if not
    /// Returns none if not in visual mode
    #[inline]
    fn get_range(&self) -> Option<(usize, usize, bool)> {
        let end = self
            .tablestate
            .selected()
            .expect("Unable to generate current table: The current row is somehow none.");
        match self.mode {
            VisualMode::Off => None,
            VisualMode::Select(start) => {
                if start < end {
                    Some((start, end, true))
                } else {
                    Some((end, start, true))
                }
            }
            VisualMode::Deselect(start) => {
                if start < end {
                    Some((start, end, false))
                } else {
                    Some((end, start, false))
                }
            }
        }
    }
    /// Based on the current temporary selection, return all the selected items as reference
    /// Returns None if the current mode is not [`VisualMode::Select`]
    #[inline]
    pub fn get_temp_selection(&self) -> Option<&[T]> {
        if let Some((start, end, is_select)) = self.get_range() {
            if is_select {
                Some(&self.items[start..=end])
            } else {
                None
            }
        } else {
            None
        }
    }
    /// Get the item the cursor is on top of
    #[inline]
    pub fn get_current(&self) -> &T {
        let Some(current) = self.tablestate.selected() else {
            panic!("Invalid cursor location!");
        };
        &self.items[current]
    }
    /// Get all selected items into a vector
    #[inline]
    pub fn get_current_selection(&self) -> Vec<&T> {
        let items: Vec<&T> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| if self.selected[i] { Some(item) } else { None })
            .collect();
        if items.is_empty() {
            vec![self.get_current()]
        } else {
            items
        }
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

    /// Reset all overall selection
    #[inline]
    pub fn reset(&mut self) {
        self.selected = vec![false; self.items.len()];
        self.comp = self.gen_table();
    }

    /// Disable visual mode for the current table
    /// If apply is true, then the current temporary selection is added to the overall selection
    pub fn disable_visual(&mut self, apply: bool) {
        if apply {
            let end = self
                .tablestate
                .selected()
                .expect("Unable to apply selection: The current row is somehow none.");

            if let VisualMode::Select(start) = self.mode {
                let (a, b) = if start < end {
                    (start, end)
                } else {
                    (end, start)
                };
                for i in a..=b {
                    self.selected[i] = true;
                }
            } else if let VisualMode::Deselect(start) = self.mode {
                let (a, b) = if start < end {
                    (start, end)
                } else {
                    (end, start)
                };
                for i in a..=b {
                    self.selected[i] = false;
                }
            }
        };
        self.mode = VisualMode::Off;
        self.comp = self.gen_table();
    }
}

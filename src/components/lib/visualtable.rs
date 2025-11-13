use color_eyre::eyre::Result;
use ratatui::{
    layout::Constraint,
    prelude::Rect,
    style::Stylize,
    widgets::{Row, Table, TableState},
    Frame,
};

use crate::{
    action::{
        useraction::{Common, Normal, UserAction, Visual},
        Action,
    },
    app::Mode,
    components::traits::{fullcomp::FullComp, renderable::Renderable},
};

/// Struct that contains the state of the current temporary selection
/// [`TempSelection::start`] is guaranteed to be smaller than or equal to [`TempSelection::end`]
/// If [`TempSelection::is_select`] is true, then the current selection mode is
/// [`VisualMode::Select`].
pub struct TempSelection {
    pub start: usize,
    pub end: usize,
    pub is_select: bool,
}

/// Various types of selections that can happen in visual table
pub enum SelectionType {
    /// The table was not in visual mode, and nothing was selected. Defaulting to the item the
    /// cursor is currently on top of.
    Single(usize),
    /// The table is currently in visual select mode.
    TempSelection(usize, usize),
    /// The table is currently not in visual mode, but some elements were selected from the
    /// previous visual mode selections.
    Selection(Vec<bool>),
    /// Nothing selected because either the cursor does not exist on the table
    None { unselect: bool },
}

impl TempSelection {
    fn new(start: usize, end: usize, is_select: bool) -> Self {
        Self {
            start,
            end,
            is_select,
        }
    }
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

/// Visual mode state tracker
/// All functions that may change the way table looks must end with the code [`self.table =
/// self.regen_table`] to ensure that the table is displayed properly.
pub struct VisualTable {
    /// Current mode of the table
    mode: VisualMode,
    /// List of all selected items
    selected: Vec<bool>,
    table_proc: fn(Table<'static>) -> Table<'static>,
    constraints: Vec<Constraint>,
    table: Table<'static>,
    tablestate: TableState,
    rows: Vec<Row<'static>>,
}

impl Renderable for VisualTable {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_stateful_widget(&self.table, area, &mut self.tablestate);
        Ok(())
    }
}

/// For consistency, do not use [`VisualTable::regen_table`] here
impl FullComp for VisualTable {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::User(ua) => {
                let cur_pos = self
                    .tablestate
                    .selected()
                    .expect("Failed to get current cursor location.");

                let action = match ua {
                    UserAction::Common(local) => match local {
                        Common::Up => {
                            self.select_previous();
                            Ok(None)
                        }
                        Common::Down => {
                            self.select_next();
                            Ok(None)
                        }
                        Common::Top => {
                            self.select_first();
                            Ok(None)
                        }
                        Common::Bottom => {
                            self.select_last();
                            Ok(None)
                        }
                        Common::ResetState => {
                            self.reset_selections();
                            Ok(None)
                        }
                        _ => Ok(None),
                    },

                    UserAction::Normal(normal) => match normal {
                        Normal::SelectMode => {
                            self.enable_visual(cur_pos, false);
                            Ok(Some(Action::ChangeMode(Mode::Visual)))
                        }
                        Normal::DeselectMode => {
                            self.enable_visual(cur_pos, true);
                            Ok(Some(Action::ChangeMode(Mode::Visual)))
                        }
                        _ => Ok(None),
                    },
                    UserAction::Visual(visual) => match visual {
                        Visual::ExitSave => {
                            self.disable_visual();
                            Ok(Some(Action::ChangeMode(Mode::Normal)))
                        }
                        Visual::ExitDiscard => {
                            self.disable_visual_discard();
                            Ok(Some(Action::ChangeMode(Mode::Normal)))
                        }
                    },
                    _ => Ok(None),
                };
                action
            }
            _ => Ok(None),
        }
    }
}

impl VisualTable {
    /// Returns true if the table is currently in Visual mode (both select and unselect)
    pub fn in_visual_mode(&self) -> bool {
        matches!(self.mode, VisualMode::Off)
    }
    /// Set all the rows with a new set of rows
    pub fn set_rows(&mut self, rows: Vec<Row<'static>>) {
        self.rows = rows;
        self.table = self.regen_table();
    }
    /// Resets the entire table, overwriting all existing rows with the new ones
    pub fn reset_rows(&mut self, rows: Vec<Row<'static>>) {
        self.selected = vec![false; rows.len()];
        self.rows = rows;
        self.table = self.regen_table();
    }
    /// Set all the rows with a new set of rows, then signal the table that there have been
    /// additional elements at the specified index
    pub fn add_rows_at(&mut self, rows: Vec<Row<'static>>, at: usize, len: usize) {
        if self.rows.is_empty() {
            self.selected = vec![false; len];
            self.rows = rows;
        } else {
            let cur = self.get_current().expect("Failed to get cursor location.");
            self.rows = rows;
            if at > self.selected.len() {
                self.selected.append(&mut vec![false; len]);
            } else {
                self.selected.splice(at..at, vec![false; len]);
            }
            if cur >= at {
                self.tablestate.select(Some(cur + len));
            }
        };
        self.table = self.regen_table();
    }

    /// Delete a row specified by the provided index
    fn delete_row(&mut self, index: usize) {
        self.rows.remove(index);
        self.selected.remove(index);
        self.table = self.regen_table();
    }

    /// Delete selected items
    pub fn delete(&mut self) -> (SelectionType, Option<Action>) {
        let (selection, action) = self.get_selection_reset();
        match selection {
            SelectionType::Single(index) => {
                self.delete_row(index);
            }
            SelectionType::TempSelection(_, _) => {
                self.delete_temp_selection();
            }
            SelectionType::Selection(_) => {
                self.delete_selection();
            }
            _ => {}
        };
        (selection, action)
    }

    /// Delete a temporarily-selected region and return the range that got deleted.
    fn delete_temp_selection(&mut self) -> Option<TempSelection> {
        if let Some(range) = self.get_temp_range() {
            if range.is_select {
                self.rows.drain(range.start..=range.end);
                self.selected.drain(range.start..=range.end);
                self.table = self.regen_table();
            };
            Some(range)
        } else {
            None
        }
    }

    /// Delete selected rows, and return the number of rows affected
    fn delete_selection(&mut self) -> usize {
        let mut count = 0;
        self.rows = self
            .rows
            .clone()
            .into_iter()
            .enumerate()
            .filter(|(idx, _)| {
                // Selected items should be deleted and should return false
                let selected = self.selected[*idx];
                if selected {
                    count += 1;
                }
                !selected
            })
            .map(|(_, row)| row)
            .collect();
        self.selected = vec![false; self.rows.len()];
        self.table = self.regen_table();
        count
    }

    /// Same as [`Self::get_selection`], except the selections are reset.
    pub fn get_selection_reset(&mut self) -> (SelectionType, Option<Action>) {
        let selection = self.get_selection();
        match selection {
            SelectionType::TempSelection(_, _) => {
                self.disable_visual_discard();
                (selection, Some(Action::ChangeMode(Mode::Normal)))
            }
            SelectionType::Selection(_) => {
                self.reset_selections();
                (selection, None)
            }
            _ => (selection, None),
        }
    }

    /// Get current selection.
    /// Priorities are as follows:
    /// 1. If the table is currently in visual mode, return the start and end index of the current
    ///    temporary selection. Return None if the current mode is visual deselect mode.
    /// 2. If there are selected items, return them in the form of boolean array.
    /// 3. If there are no rows selected, return the item selected by the cursor. Return None if
    ///    there is no cursor present.
    pub fn get_selection(&self) -> SelectionType {
        if let Some(range) = self.get_temp_range() {
            if range.is_select {
                return SelectionType::TempSelection(range.start, range.end);
            } else {
                return SelectionType::None { unselect: true };
            }
        } else {
            let mut count = 0;
            for b in &self.selected {
                if *b {
                    count += 1;
                }
            }

            if count == 0 {
                match self.get_current() {
                    Some(index) => SelectionType::Single(index),
                    None => SelectionType::None { unselect: false },
                }
            } else {
                SelectionType::Selection(self.selected.clone())
            }
        }
    }

    #[inline]
    fn select_previous(&mut self) {
        self.tablestate.select_previous();
        self.table = self.regen_table();
    }

    #[inline]
    fn select_next(&mut self) {
        self.tablestate.select_next();
        self.table = self.regen_table();
    }

    #[inline]
    fn select_last(&mut self) {
        self.tablestate.select_last();
        self.table = self.regen_table();
    }

    #[inline]
    fn select_first(&mut self) {
        self.tablestate.select_first();
        self.table = self.regen_table();
    }

    /// Consumes the given rows to create a new set of rows with (temporary) selection indicators
    fn gen_rows(
        temp: Option<TempSelection>,
        rows: Vec<Row<'static>>,
        selected: &[bool],
    ) -> Vec<Row<'static>> {
        let iter = rows.into_iter().enumerate();
        match temp {
            Some(t) => iter
                .map(|(i, mut row)| {
                    row = if i <= t.end && i >= t.start {
                        row.reversed()
                    } else {
                        row
                    };
                    if selected[i] {
                        row.green()
                    } else {
                        row
                    }
                })
                .collect(),
            None => iter
                .map(|(i, row)| if selected[i] { row.green() } else { row })
                .collect(),
        }
    }

    /// Regenerate the table so that its look matches the table's internal state
    fn regen_table(&self) -> Table<'static> {
        Self::gen_table(
            &self.constraints,
            self.rows.clone(),
            self.get_temp_range(),
            &self.get_current_selection(),
            self.table_proc,
        )
    }

    /// Generate a table so that its look matches the table's internal state.
    fn gen_table(
        constraints: &Vec<Constraint>,
        rows: Vec<Row<'static>>,
        temp: Option<TempSelection>,
        selected: &[bool],
        table_proc: fn(Table<'static>) -> Table<'static>,
    ) -> Table<'static> {
        let comp = Table::new(Self::gen_rows(temp, rows, selected), constraints);
        (table_proc)(comp)
    }

    pub fn new(
        rows: Vec<Row<'static>>,
        constraints: Vec<Constraint>,
        table_proc: fn(Table<'static>) -> Table<'static>,
    ) -> Self {
        let selected = vec![false; rows.len()];
        let table = Self::gen_table(&constraints, rows.clone(), None, &selected, table_proc);
        Self {
            mode: VisualMode::Off,
            rows,
            selected,
            table_proc,
            constraints,
            table,
            tablestate: TableState::new().with_selected(Some(0)),
        }
    }
    /// Enters visual mode
    /// If [`deselect`] is true, then [`VisualMode::Deselect`] is used instead
    pub fn enable_visual(&mut self, current: usize, deselect: bool) {
        self.mode = if deselect {
            VisualMode::Deselect(current)
        } else {
            VisualMode::Select(current)
        };
        self.table = self.regen_table();
    }
    /// Get temporary selection
    /// Returns index of the start of selection and end, and boolean that indicates selection mode
    /// First index is guaranteed to be smaller than the second
    /// If true, then the current selection is [`VisualMode::Select`], false if not
    /// Returns none if not in visual mode
    #[inline]
    fn get_range(&self, end: usize) -> Option<(usize, usize, bool)> {
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
    /// Returns current temporary selection
    /// First index is guaranteed to be smaller than the second
    /// If the third value is true, then the table is in select mode. If not, the table is in
    /// deselect mode.
    #[inline]
    fn get_temp_range(&self) -> Option<TempSelection> {
        let end = self
            .get_current()
            .expect("Cannot find the cursor location!");
        if let Some((start, end, is_select)) = self.get_range(end) {
            if start > end {
                Some(TempSelection::new(end, start, is_select))
            } else {
                Some(TempSelection::new(start, end, is_select))
            }
        } else {
            None
        }
    }
    /// Get a reference to the selection toggle list
    #[inline]
    fn get_current_selection(&self) -> &[bool] {
        &self.selected
    }

    /// Reset all overall selection
    #[inline]
    pub fn reset_selections(&mut self) {
        self.selected = vec![false; self.selected.len()];
        self.table = self.regen_table();
    }
    #[inline]
    pub fn get_current(&self) -> Option<usize> {
        self.tablestate.selected()
    }

    /// Disable visual mode for the current table
    /// The current temporary selection is added to the overall selection
    pub fn disable_visual(&mut self) {
        let end = self
            .get_current()
            .expect("Cannot find the cursor location!");
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
        };
        self.mode = VisualMode::Off;
        self.table = self.regen_table();
    }
    /// Disable visual mode for the current table
    /// The current temporary selection is not added to the overall selection
    pub fn disable_visual_discard(&mut self) {
        self.mode = VisualMode::Off;
        self.table = self.regen_table();
    }
}

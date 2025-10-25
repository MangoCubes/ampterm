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
    components::traits::component::Component,
    playerworker::player::QueueLocation,
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

/// For consistency, do not use [`VisualTable::regen_table`] here
impl Component for VisualTable {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_stateful_widget(&self.table, area, &mut self.tablestate);
        Ok(())
    }

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
                        _ => Ok(None),
                    },
                };
                action
            }
            _ => Ok(None),
        }
    }
}

impl VisualTable {
    /// Resets the entire table, overwriting all existing rows with the new ones
    pub fn reset_rows(&mut self, rows: Vec<Row<'static>>) {
        self.selected = vec![false; rows.len()];
        self.rows = rows;
        self.table = self.regen_table();
    }
    /// Adds a number of rows at a specific index
    pub fn add_rows_at(&mut self, mut rows: Vec<Row<'static>>, at: usize, pos: QueueLocation) {
        if self.rows.is_empty() {
            self.rows = rows;
        } else {
            let len = rows.len();
            match pos {
                QueueLocation::Front => {
                    self.rows.splice(at..at, rows);
                    self.selected.splice(at..at, vec![false; len]);
                }
                QueueLocation::Next => {
                    self.rows.splice((at + 1)..(at + 1), rows);
                    self.selected.splice(at..at, vec![false; len]);
                }
                QueueLocation::Last => {
                    self.rows.append(&mut rows);
                    self.selected.extend(vec![false; len]);
                }
            };
        };
        self.table = self.regen_table();
    }
    pub fn add_rows(&mut self, rows: Vec<Row<'static>>, pos: QueueLocation) {
        self.add_rows_at(
            rows,
            self.get_current().expect("Failed to get cursor location."),
            pos,
        );
    }
    pub fn delete_temp_selection(&mut self) {
        if let Some(range) = self.get_temp_range() {
            self.rows.drain(range.start..=range.end);
            self.selected.drain(range.start..=range.end);
            self.table = self.regen_table();
        }
    }
    pub fn delete_selection(&mut self) {
        self.rows = self
            .rows
            .clone()
            .into_iter()
            .enumerate()
            .filter(|(idx, _)| self.selected[*idx])
            .map(|(_, row)| row)
            .collect();
        self.selected = vec![false; self.rows.len()];
        self.table = self.regen_table();
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
    pub fn regen_table(&self) -> Table<'static> {
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
    pub fn get_temp_range(&self) -> Option<TempSelection> {
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
    pub fn get_current_selection(&self) -> &[bool] {
        &self.selected
    }
    /// Reset all overall selection
    #[inline]
    pub fn reset(&mut self) {
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

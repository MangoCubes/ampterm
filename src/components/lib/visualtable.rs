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
};

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
                            self.tablestate.select_previous();
                            Ok(None)
                        }
                        Common::Down => {
                            self.tablestate.select_next();
                            Ok(None)
                        }
                        Common::Top => {
                            self.tablestate.select_first();
                            Ok(None)
                        }
                        Common::Bottom => {
                            self.tablestate.select_last();
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
                            self.disable_visual(cur_pos);
                            Ok(Some(Action::ChangeMode(Mode::Normal)))
                        }
                        Visual::ExitDiscard => {
                            self.disable_visual_discard();
                            Ok(Some(Action::ChangeMode(Mode::Normal)))
                        }
                        _ => Ok(None),
                    },
                };
                self.table = self.regen_table();
                action
            }
            _ => Ok(None),
        }
    }
}

impl VisualTable {
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
    pub fn regen_table(&self) -> Table<'static> {
        Self::gen_table(
            &self.constraints,
            self.rows.clone(),
            self.get_temp_range(),
            &self.get_current_selection(),
            self.table_proc,
        )
    }
    fn gen_table(
        constraints: &Vec<Constraint>,
        rows: Vec<Row<'static>>,
        temp: Option<TempSelection>,
        selected: &[bool],
        table_proc: fn(Table<'static>) -> Table<'static>,
    ) -> Table<'static> {
        let comp = Table::new(rows, constraints);
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
    }
    #[inline]
    pub fn get_current(&self) -> Option<usize> {
        self.tablestate.selected()
    }

    /// Disable visual mode for the current table
    /// The current temporary selection is added to the overall selection
    pub fn disable_visual(&mut self, end: usize) {
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
    }
    /// Disable visual mode for the current table
    /// The current temporary selection is not added to the overall selection
    pub fn disable_visual_discard(&mut self) {
        self.mode = VisualMode::Off;
    }
}

use ratatui::{
    layout::Constraint,
    prelude::Rect,
    style::Stylize,
    widgets::{Row, Table, TableState},
    Frame,
};

use crate::{
    action::{
        action::{Action, Mode},
        localaction::ListAction,
    },
    components::traits::{
        handlekeyseq::{HandleKeySeq, KeySeqResult},
        renderable::Renderable,
    },
    config::{keybindings::KeyBindings, Config},
    helper::selection::ModifiableList,
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
pub enum VisualSelection {
    /// The table was not in visual mode, and nothing was selected. Defaulting to the item the
    /// cursor is currently on top of.
    Single(usize),
    /// The table is currently in visual select mode.
    TempSelection(usize, usize),
    /// The table is currently not in visual mode, but some elements were selected from the
    /// previous visual mode selections.
    Selection(Vec<RowState>),
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

#[derive(Clone)]
pub struct RowState {
    pub visible: bool,
    pub selected: bool,
    highlight: bool,
}

impl Default for RowState {
    fn default() -> Self {
        Self {
            visible: true,
            selected: false,
            highlight: false,
        }
    }
}

/// Visual mode state tracker
/// All functions that may change the way table looks must end with the code [`self.table =
/// self.regen_table`] to ensure that the table is displayed properly.
pub struct VisualTable {
    /// Current mode of the table
    mode: VisualMode,
    /// List of all selected items
    state: ModifiableList<RowState>,
    table_proc: fn(Table<'static>) -> Table<'static>,
    constraints: Vec<Constraint>,
    table: Table<'static>,
    tablestate: TableState,
    rows: ModifiableList<Row<'static>>,
    binds: KeyBindings<ListAction>,
    visual_binds: KeyBindings<ListAction>,
}

impl Renderable for VisualTable {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(&self.table, area, &mut self.tablestate);
    }
}

impl HandleKeySeq<ListAction> for VisualTable {
    fn get_name(&self) -> &str {
        "List"
    }
    fn handle_local_action(&mut self, action: ListAction) -> KeySeqResult {
        let cur_pos = match self.tablestate.selected() {
            Some(i) => i,
            None => {
                return KeySeqResult::NoActionNeeded;
            }
        };
        match action {
            ListAction::ExitSave => {
                self.disable_visual_save();
                KeySeqResult::ActionNeeded(Action::ChangeMode(Mode::Normal))
            }
            ListAction::ExitDiscard => {
                self.disable_visual_discard();
                KeySeqResult::ActionNeeded(Action::ChangeMode(Mode::Normal))
            }
            ListAction::Up => {
                self.select_previous();
                KeySeqResult::NoActionNeeded
            }
            ListAction::Down => {
                self.select_next();
                KeySeqResult::NoActionNeeded
            }
            ListAction::Top => {
                self.select_first();
                KeySeqResult::NoActionNeeded
            }
            ListAction::Bottom => {
                self.select_last();
                KeySeqResult::NoActionNeeded
            }
            ListAction::ResetSelection => {
                self.reset_selections();
                KeySeqResult::NoActionNeeded
            }
            ListAction::SelectMode => {
                self.enable_visual(cur_pos, false);
                KeySeqResult::ActionNeeded(Action::ChangeMode(Mode::Visual))
            }
            ListAction::DeselectMode => {
                self.enable_visual(cur_pos, true);
                KeySeqResult::ActionNeeded(Action::ChangeMode(Mode::Visual))
            }
        }
    }

    fn get_keybinds(&self) -> &KeyBindings<ListAction> {
        if matches!(self.mode, VisualMode::Off) {
            &self.binds
        } else {
            &self.visual_binds
        }
    }
}

impl VisualTable {
    /// Set all the rows with a new set of rows
    pub fn set_rows(&mut self, rows: Vec<Row<'static>>) {
        self.rows = ModifiableList::new(rows);
        self.table = self.regen_table();
    }

    /// Function that should be called if the cursor position is missing
    pub fn bump_cursor_pos(&mut self) {
        if self.rows.len() != 0 && self.tablestate.selected() == None {
            self.tablestate.select(Some(0));
        }
    }

    /// This function is intended to be called whenever new rows are added to the table. However,
    /// the table must be regenerated in full every update because the new item may affect other
    /// rows too. (Example: "Play now" action causes the item that was being played to stop, and
    /// that row needs to be updated)
    /// 1. Receive a list, and overwrite the whole table with it
    /// 2. With the information given through [`at`] and [`len`], update the current selection so
    ///    that it correctly reflects the new
    pub fn add_rows_at(&mut self, rows: Vec<Row<'static>>, at: usize, len: usize) {
        self.state.add_rows_at(vec![RowState::default(); len], at);
        if !self.rows.is_empty() {
            if let Some(cur) = self.get_current() {
                if cur >= at {
                    self.tablestate.select(Some(cur + len));
                }
            }
        };
        self.set_rows(rows);
    }

    /// Same as [`Self::get_selection`], except the selections are reset.
    pub fn get_selection_reset(&mut self) -> (VisualSelection, Option<Action>) {
        let selection = self.get_selection();
        match selection {
            VisualSelection::TempSelection(_, _) => {
                self.disable_visual_discard();
                (selection, Some(Action::ChangeMode(Mode::Normal)))
            }
            VisualSelection::Selection(_) => {
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
    pub fn get_selection(&self) -> VisualSelection {
        if let Some(range) = self.get_temp_range() {
            if range.is_select {
                return VisualSelection::TempSelection(range.start, range.end);
            } else {
                return VisualSelection::None { unselect: true };
            }
        } else {
            let mut count = 0;
            for b in &self.state.0 {
                if b.selected {
                    count += 1;
                }
            }

            if count == 0 {
                match self.get_current() {
                    Some(index) => VisualSelection::Single(index),
                    None => VisualSelection::None { unselect: false },
                }
            } else {
                VisualSelection::Selection(self.state.clone())
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
        selected: &[RowState],
    ) -> Vec<Row<'static>> {
        let iter = rows
            .into_iter()
            .zip(selected)
            .enumerate()
            .filter(|(_, (_, state))| state.visible);
        match temp {
            Some(t) => iter
                .map(|(i, (mut row, state))| {
                    row = if i <= t.end && i >= t.start {
                        row.reversed()
                    } else {
                        row
                    };
                    if state.selected {
                        row.green()
                    } else {
                        row
                    }
                })
                .collect(),
            None => iter
                .map(|(_, (row, state))| if state.selected { row.green() } else { row })
                .collect(),
        }
    }

    /// Regenerate the table so that its look matches the table's internal state
    fn regen_table(&self) -> Table<'static> {
        Self::gen_table(
            &self.constraints,
            self.rows.0.clone(),
            self.get_temp_range(),
            &self.state,
            self.table_proc,
        )
    }

    /// Generate a table so that its look matches the table's internal state.
    fn gen_table(
        constraints: &Vec<Constraint>,
        rows: Vec<Row<'static>>,
        temp: Option<TempSelection>,
        selected: &[RowState],
        table_proc: fn(Table<'static>) -> Table<'static>,
    ) -> Table<'static> {
        let comp = Table::new(Self::gen_rows(temp, rows, selected), constraints);
        (table_proc)(comp)
    }

    pub fn new(
        config: Config,
        rows: Vec<Row<'static>>,
        constraints: Vec<Constraint>,
        table_proc: fn(Table<'static>) -> Table<'static>,
    ) -> Self {
        let selected = ModifiableList::new(vec![RowState::default(); rows.len()]);
        let table = Self::gen_table(&constraints, rows.clone(), None, &selected, table_proc);
        Self {
            mode: VisualMode::Off,
            rows: ModifiableList::new(rows),
            state: selected,
            table_proc,
            constraints,
            table,
            tablestate: TableState::new().with_selected(Some(0)),
            binds: config.local.list,
            visual_binds: config.local.list_visual,
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
        let Some(end) = self.get_current() else {
            return None;
        };
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

    /// Reset all overall selection
    #[inline]
    pub fn reset_selections(&mut self) {
        self.state = ModifiableList::new(vec![RowState::default(); self.state.0.len()]);
        self.table = self.regen_table();
    }
    #[inline]
    pub fn get_current(&self) -> Option<usize> {
        self.tablestate.selected()
    }

    #[inline]
    pub fn set_visibility(&mut self, visible: &[bool]) {
        if self.state.len() != visible.len() {
            panic!("Received invalid visibility vector! It's supposed to be as long as the number of elements.");
        };
        self.state
            .iter_mut()
            .zip(visible)
            .for_each(|(state, new)| state.visible = *new);
    }

    #[inline]
    pub fn reset_visibility(&mut self) {
        self.state.iter_mut().for_each(|state| state.visible = true);
    }

    #[inline]
    pub fn set_highlight(&mut self, highlight: &[bool]) {
        if self.state.len() != highlight.len() {
            panic!("Received invalid highlight vector! It's supposed to be as long as the number of elements.");
        };
        self.state
            .iter_mut()
            .zip(highlight)
            .for_each(|(state, new)| state.highlight = *new);
    }

    #[inline]
    pub fn reset_highlight(&mut self) {
        self.state
            .iter_mut()
            .for_each(|state| state.highlight = false);
    }

    /// Disable visual mode for the current table
    /// The current temporary selection is added to the overall selection
    pub fn disable_visual_save(&mut self) {
        let Some(end) = self.get_current() else {
            self.mode = VisualMode::Off;
            self.table = self.regen_table();
            return;
        };
        if let VisualMode::Select(start) = self.mode {
            let (a, b) = if start < end {
                (start, end)
            } else {
                (end, start)
            };
            for i in a..=b {
                if self.state[i].visible {
                    self.state[i].selected = true;
                }
            }
        } else if let VisualMode::Deselect(start) = self.mode {
            let (a, b) = if start < end {
                (start, end)
            } else {
                (end, start)
            };
            for i in a..=b {
                if self.state[i].visible {
                    self.state[i].selected = false;
                }
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

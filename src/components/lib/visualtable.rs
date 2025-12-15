use derive_deref::{Deref, DerefMut};
use ratatui::{
    layout::Constraint,
    prelude::Rect,
    style::Stylize,
    widgets::{Row, Table, TableState},
    Frame,
};

/// Filter applied index are indexes that are counted from the first element, skipping the ones
/// that are hidden ([`RowState::visible`]). If there are 5 elements, and 1 and 2 has
/// [`RowState::visible`] set to false, then the 5th element's index is 2 (4 - 2).
#[derive(Deref, DerefMut, Clone, Copy)]
struct FilterAppliedIndex(usize);

impl FilterAppliedIndex {
    pub fn from(idx: usize, state: &Vec<RowState>) -> Self {
        let mut real = idx;
        let len = state.len();
        let mut last_visible = 0;
        for i in 0..len {
            if state[i].visible {
                if real == 0 {
                    return FilterAppliedIndex(i);
                } else {
                    real -= 1;
                    last_visible = i;
                }
            }
        }
        FilterAppliedIndex(last_visible)
    }
    pub fn to_user(&self, state: &Vec<RowState>) -> usize {
        let mut visible_count = 0;
        for i in (0..self.0).rev() {
            if state[i].visible {
                visible_count += 1;
            }
        }
        visible_count
    }
}

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
    start: FilterAppliedIndex,
    end: FilterAppliedIndex,
    pub is_select: bool,
}

/// Various types of selections that can happen in visual table
pub enum VisualSelection {
    /// The table was not in visual mode, and nothing was selected. Defaulting to the item the
    /// cursor is currently on top of.
    Single(usize),
    /// The table is currently not in visual mode, but some elements were selected from the
    /// previous visual mode selections.
    Multiple { temp: bool, map: Vec<bool> },
    /// Nothing selected because either the cursor does not exist on the table
    None,
}

impl TempSelection {
    fn new(start: FilterAppliedIndex, end: FilterAppliedIndex, is_select: bool) -> Self {
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
    Select(FilterAppliedIndex),
    // Visual mode enabled (deselect selected region)
    // The number represent the start of the region
    Deselect(FilterAppliedIndex),
}

#[derive(Clone)]
pub struct RowState {
    pub visible: bool,
    pub selected: bool,
    pub highlight: bool,
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
                self.enable_visual(FilterAppliedIndex::from(cur_pos, &self.state), false);
                KeySeqResult::ActionNeeded(Action::ChangeMode(Mode::Visual))
            }
            ListAction::DeselectMode => {
                self.enable_visual(FilterAppliedIndex::from(cur_pos, &self.state), true);
                KeySeqResult::ActionNeeded(Action::ChangeMode(Mode::Visual))
            }
            ListAction::SearchNext => {
                self.jump_next();
                KeySeqResult::NoActionNeeded
            }
            ListAction::SearchPrev => {
                self.jump_prev();
                KeySeqResult::NoActionNeeded
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
            VisualSelection::Multiple { temp, map: _ } => {
                if temp {
                    self.disable_visual_discard();
                    (selection, Some(Action::ChangeMode(Mode::Normal)))
                } else {
                    self.reset_selections();
                    (selection, None)
                }
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
                let map = self
                    .state
                    .iter()
                    .enumerate()
                    .map(|(i, r)| r.visible && i >= range.start.0 && i <= range.end.0)
                    .collect();
                return VisualSelection::Multiple { temp: true, map };
            } else {
                return VisualSelection::None;
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
                    Some(index) => {
                        VisualSelection::Single(FilterAppliedIndex::from(index, &self.state).0)
                    }
                    None => VisualSelection::None,
                }
            } else {
                let map = self.state.iter().map(|r| r.selected).collect();
                VisualSelection::Multiple { temp: false, map }
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
                    row = if i <= t.end.0 && i >= t.start.0 {
                        row.reversed()
                    } else {
                        row
                    };
                    if state.selected {
                        row.green()
                    } else if state.highlight {
                        row.yellow()
                    } else {
                        row
                    }
                })
                .collect(),
            None => iter
                .map(|(_, (row, state))| {
                    if state.selected {
                        row.green()
                    } else if state.highlight {
                        row.yellow()
                    } else {
                        row
                    }
                })
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
    fn enable_visual(&mut self, current: FilterAppliedIndex, deselect: bool) {
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
    fn get_range(
        &self,
        end: FilterAppliedIndex,
    ) -> Option<(FilterAppliedIndex, FilterAppliedIndex, bool)> {
        match self.mode {
            VisualMode::Off => None,
            VisualMode::Select(start) => {
                if start.0 < end.0 {
                    Some((start, end, true))
                } else {
                    Some((end, start, true))
                }
            }
            VisualMode::Deselect(start) => {
                if start.0 < end.0 {
                    Some((start, end, false))
                } else {
                    Some((end, start, false))
                }
            }
        }
    }

    fn jump_prev(&mut self) -> bool {
        let idx = FilterAppliedIndex::from(self.get_current().unwrap_or(0), &self.state);
        for i in (0..idx.0).rev() {
            if self.state[i].visible && self.state[i].highlight {
                let idx = FilterAppliedIndex(i).to_user(&self.state);
                self.tablestate.select(Some(idx));
                return true;
            }
        }
        false
    }

    fn jump_next(&mut self) -> bool {
        let idx = FilterAppliedIndex::from(self.get_current().unwrap_or(0), &self.state);
        for i in (idx.0 + 1)..self.state.len() {
            if self.state[i].visible && self.state[i].highlight {
                let idx = FilterAppliedIndex(i).to_user(&self.state);
                self.tablestate.select(Some(idx));
                return true;
            }
        }
        false
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
        if let Some((start, end, is_select)) =
            self.get_range(FilterAppliedIndex::from(end, &self.state))
        {
            Some(TempSelection::new(start, end, is_select))
        } else {
            None
        }
    }

    /// Reset all overall selection
    #[inline]
    pub fn reset_selections(&mut self) {
        self.state.iter_mut().for_each(|r| r.selected = false);
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
        self.table = self.regen_table();
    }

    #[inline]
    pub fn reset_visibility(&mut self) {
        self.state.iter_mut().for_each(|state| state.visible = true);
        self.table = self.regen_table();
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
        self.table = self.regen_table();
    }

    #[inline]
    pub fn reset_highlight(&mut self) {
        self.state
            .iter_mut()
            .for_each(|state| state.highlight = false);
        self.table = self.regen_table();
    }

    /// Disable visual mode for the current table
    /// The current temporary selection is added to the overall selection
    pub fn disable_visual_save(&mut self) {
        let Some(temp) = self.get_temp_range() else {
            panic!("Attempted to exit visual mode when not in visual mode!");
        };
        for i in temp.start.0..=temp.end.0 {
            if self.state[i].visible {
                self.state[i].selected = temp.is_select;
            }
        }
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

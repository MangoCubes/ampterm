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
pub struct VisualState {
    /// Current mode of the table
    mode: VisualMode,
    /// List of all selected items
    selected: Vec<bool>,
}

impl VisualState {
    pub fn new(len: usize) -> Self {
        let selected = vec![false; len];
        Self {
            mode: VisualMode::Off,
            selected,
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
    pub fn get_temp_selection(&self, end: usize) -> Option<(usize, usize, bool)> {
        if let Some((start, end, is_select)) = self.get_range(end) {
            if start > end {
                Some((end, start, is_select))
            } else {
                Some((start, end, is_select))
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

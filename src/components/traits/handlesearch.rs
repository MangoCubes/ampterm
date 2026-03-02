use crate::components::traits::renderable::Renderable;

/// Any component that implements this trait implies that the component can use the given search
/// conditions to highlight items that matches the given condition
pub trait HandleSearch: Renderable {
    /// Temporarily sets the highlight so that all items that matches the conditions are
    /// highlighted
    fn test_search(&mut self, search: String);
    /// Executed when the user confirms the search by pressing enter
    fn confirm_search(&mut self, search: String) -> Action {
        self.apply_search(search);
        self.state = State::Nothing;
        Action::ChangeMode(Mode::Normal)
    }

    /// Executed whent the search is cancelled or removed
    fn clear_search(&mut self) -> Action {
        if let State::Searching(_, idx) = self.state {
            self.table.set_position(idx);
        };
        self.state = State::Nothing;
        self.search = None;
        self.table.reset_highlight();
        self.table.bump_cursor_pos();
        Action::ChangeMode(Mode::Normal)
    }
}

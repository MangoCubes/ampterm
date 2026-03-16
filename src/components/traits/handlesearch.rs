use crate::{action::action::SearchType, components::traits::renderable::Renderable};

/// When user starts searching, the program needs to store the current location of the cursor so
/// that when the user cancels search, it jumps back to where they were prior to search
/// This is fulfiled with [`HandleSearch::init_search`]
///
/// When user types a character, the program should move the cursor to the next matching item
/// This is fulfiled with [`HandleSearch::test_search`], even if the keyword is empty
/// As a result, [`HandleSearch::test_search`] also handles clearing search
///
/// When user presses enter, it should do nothing
///
/// When user presses escape, the search should revert to the previous one (or none if there wasn't
/// any), and move the cursor back to where it was before the search
/// This is fulfiled with combination of [`HandleSearch::test_search`].

/// Any component that implements this trait implies that the component can use the given search
/// conditions to highlight items that matches the given condition
pub trait HandleSearch: Renderable {
    /// Executed when search is initialised
    /// If return value is false, it means the currently focused component does not support search,
    /// and the search is aborted.
    /// This would be a good time to store the location of the cursor before search is applied
    fn init_search(&mut self) -> bool;
    /// Temporarily sets the highlight so that all items that matches the conditions are
    /// highlighted
    /// If revert is true, then this indicates that the search is a revert, and cursor position
    /// should be reverted.
    fn test_search(&mut self, search: String, stype: SearchType);
}

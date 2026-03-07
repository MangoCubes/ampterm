use crate::{
    action::action::{Action, Mode},
    components::traits::renderable::Renderable,
};

/// Any component that implements this trait implies that the component can use the given search
/// conditions to highlight items that matches the given condition
pub trait HandleSearch: Renderable {
    /// Executed when search is initialised
    /// This would be a good time to store the location of the cursor before search is applied
    fn init_search(&mut self);
    /// Temporarily sets the highlight so that all items that matches the conditions are
    /// highlighted
    fn test_search(&mut self, search: String);
    /// Executed when the user confirms the search by pressing enter
    fn confirm_search(&mut self, search: String) -> Action {
        self.test_search(search);
        Action::ChangeMode(Mode::Normal)
    }

    /// Executed whent the search is cancelled or removed
    fn clear_search(&mut self) -> Action;
}

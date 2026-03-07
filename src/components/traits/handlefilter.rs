use crate::{action::action::Action, components::traits::renderable::Renderable};

/// Any component that implements this trait implies that the component can use the given filter to
/// hide unwanted contents
pub trait HandleFilter: Renderable {
    /// Applies filter, hiding anything that do not fit the criteria
    fn set_filter(&mut self, filter: String) -> Action;
    /// Removes the filter, revealing all elements that were hidden
    fn reset_filter(&mut self) -> Action;
    /// Revert to the previous state, keeping the filter
    fn exit_filter(&mut self) -> Action;
}

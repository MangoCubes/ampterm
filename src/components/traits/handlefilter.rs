use crate::components::traits::renderable::Renderable;

/// Any component that implements this trait implies that the component can use the given filter to
/// hide unwanted contents
pub trait HandleFilter: Renderable {
    fn init_filter(&mut self) -> bool {
        false
    }
    /// Applies filter, hiding anything that do not fit the criteria
    fn set_filter(&mut self, filter: String);
}

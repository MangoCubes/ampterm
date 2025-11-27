use crate::components::traits::renderable::Renderable;

/// If a component has a focusable trait, this indicates that the component has two states:
/// Focused and not focused.
pub trait Focusable: Renderable {
    fn set_enabled(&mut self, enable: bool);
}

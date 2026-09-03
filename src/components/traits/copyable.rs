use crate::components::traits::renderable::Renderable;

/// If a component has this trait, then user can enter CopyToClipboard action to copy a string into
/// the clipboard
pub trait Copyable: Renderable {
    fn get_copyable_item(&self) -> Option<String>;
}

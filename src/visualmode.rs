use crate::components::Component;

pub trait VisualMode<T>: Component {
    // Enters visual mode for multiple selection
    fn start_visual_mode(&mut self) -> bool;

    // Exits visual mode without resetting the current selection
    // This operation preserves current selection.
    // Refer to reset_visual_mode for resetting the selection.
    fn end_visual_mode(&mut self) -> bool;

    // Exits visual mode and resets current selection
    // This function may be called outside the visual mode to reset the current selection.
    fn reset_visual_mode(&mut self) -> bool;

    // Gets the selected region
    fn get_selection(&mut self) -> Vec<T>;
}

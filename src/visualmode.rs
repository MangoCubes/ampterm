use crate::components::Component;
use std::collections::HashSet;
use std::hash::Hash;

// Visual mode have two storages
// temp: Items selected by the current visual mode
// (unmarked): Items selected by all visual modes since the last reset
pub trait VisualMode<T: Eq + Hash + Clone>: Component {
    // Check if the current mode is visual mode
    fn is_visual(self) -> bool;
    // Enable/disable visual mode
    fn set_visual(&mut self, to: bool);

    // Toggle visual mode
    // Add temporarily selected items into the list of selected items, unless they are already
    // added, in which case they are removed instead
    fn toggle_visual_mode(&mut self) {
        if self.is_visual() {
            let old = self.get_selection();
            let new = self.get_temp_selection();
            let merged = if let Some(new_selected) = new {
                if let Some(old_selected) = old {
                    // Both exists
                    let merged: HashSet<T> = old_selected
                        .symmetric_difference(&new_selected)
                        .cloned()
                        .collect();
                    Some(merged)
                } else {
                    // Missing old selection
                    Some(new_selected)
                }
            } else {
                if let Some(old_selected) = old {
                    Some(old_selected)
                } else {
                    None
                }
            };
            self.set_selection(merged);
            self.set_temp_selection(None);
            self.set_visual(false);
        } else {
            self.set_visual(true);
        }
    }

    // Reset all selection
    fn reset_normal_selection(&mut self) {
        self.set_selection(None);
        self.set_temp_selection(None);
        self.set_visual(false);
    }

    // Gets the temporarily selected region
    fn get_temp_selection(self) -> Option<&HashSet<T>>;

    // Gets the selected region
    fn get_selection(self) -> Option<&HashSet<T>>;

    // Sets the selected region
    fn set_selection(&mut self, selection: Option<HashSet<T>>);

    // Sets the temporarily selected region
    fn set_temp_selection(&mut self, selection: Option<HashSet<T>>);
}

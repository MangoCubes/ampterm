use crate::components::Component;
use std::collections::HashSet;
use std::hash::Hash;

// Visual mode have two storages
// temp: Items selected by the current visual mode
// (unmarked): Items selected by all visual modes since the last reset
pub trait VisualMode<T: Eq + Hash + Clone>: Component {
    // Check if the current mode is visual mode
    fn is_visual(&self) -> bool;
    // Enable/disable visual mode
    // This behaviour should be defined, but not be used directly
    fn set_visual(&mut self, to: bool);

    // Change current mode
    // If the current mode and the mode to change to are the same, then nothing happens
    // Otherwise:
    //   If the current mode is visual, and we are about to change to normal mode, the current
    //   selection is saved
    //   If the current mode is normal, and we are about to change to visual mode, the mode changes
    fn set_visual_mode(&mut self, to: bool) {
        let current = self.is_visual();
        if to == current {
            return;
        };
        if self.is_visual() {
            let old = self.get_selection();
            let new = self.get_temp_selection();
            if let Some(new_selected) = new {
                if let Some(old_selected) = old {
                    // Both exists
                    let merged: HashSet<T> = old_selected
                        .symmetric_difference(&new_selected)
                        .cloned()
                        .collect();
                    self.set_selection(Some(merged));
                } else {
                    // Missing old selection
                    Some(new_selected);
                }
            } else {
                if let Some(old_selected) = old {
                    Some(old_selected);
                }
            };
            self.set_temp_selection(None);
            self.set_visual(false);
        } else {
            self.set_visual(true);
        }
    }

    // Reset all selection
    fn reset_all_selection(&mut self) {
        self.set_selection(None);
        self.set_temp_selection(None);
        self.set_visual(false);
    }

    // Gets the temporarily selected region
    fn get_temp_selection(&self) -> Option<&HashSet<T>>;

    // Gets the selected region
    fn get_selection(&self) -> Option<&HashSet<T>>;

    // Sets the selected region
    fn set_selection(&mut self, selection: Option<HashSet<T>>);

    // Sets the temporarily selected region
    fn set_temp_selection(&mut self, selection: Option<HashSet<T>>);
}

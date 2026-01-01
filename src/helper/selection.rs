use derive_deref::{Deref, DerefMut};

#[derive(Deref, DerefMut)]
pub struct ModifiableList<T: Clone>(pub Vec<T>);

pub enum Selection {
    Single(usize),
    Multiple(Vec<bool>),
}

impl<T: Clone> ModifiableList<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self(items)
    }
    /// Delete a row specified by the provided index
    fn delete_single(&mut self, index: usize) {
        self.remove(index);
    }

    /// Delete rows where selected[index] is true
    fn delete_multiple(&mut self, selected: &[bool]) {
        self.0 = self
            .iter()
            .zip(selected)
            .filter_map(|(item, &flag)| if flag { None } else { Some(item.clone()) })
            .collect();
    }

    /// This function answers the following question:
    /// "If the given selection is deleted, where should the cursor at the given index should go?"
    ///
    /// Returns the new index and whether the current item has been deleted, or returns None if
    /// all items are deleted so that there can be no valid position for the cursor to settle at.
    pub fn move_item_to(&self, selection: &Selection, idx: usize) -> Option<(usize, bool)> {
        let len = self.len();
        match selection {
            // Only one item is deleted
            Selection::Single(index) => {
                if len == 1 {
                    // ...and it's the only item left in the list.
                    None
                } else if idx == *index {
                    // ...and it's the current item.
                    if idx == 0 {
                        // Default to position 0 because there is no item before that
                        Some((0, true))
                    } else {
                        Some((idx - 1, true))
                    }
                } else {
                    // ...but it's not the current item.
                    if idx > *index {
                        // If the current item comes after the current item, move up the list by
                        // one
                        Some((idx - 1, false))
                    } else {
                        Some((idx, false))
                    }
                }
            }
            Selection::Multiple(items) => {
                // [`safe_items`] specifies the number of items that are NOT deleted by this
                // operation and comes before the current item. Essentially, it's the number of
                // items that will be left behind before the current item.
                let mut safe_items = 0;
                let deleted = items[idx];
                for i in 0..idx {
                    if !items[i] {
                        safe_items += 1;
                    }
                }
                if safe_items == 0 {
                    // If all items before the current item are deleted...
                    for i in idx..len {
                        // If any items after the current item are being left behind, that means at
                        // least one item will be left behind after the delete operation.
                        // Therefore, Some(0) is returned.
                        if !items[i] {
                            return Some((0, deleted));
                        }
                    }
                    // If we reach here, that means everything is being deleted.
                    None
                } else {
                    // This means that some items before the current item are left behind. Return
                    // the last one amongst the ones that are not deleted.
                    Some((safe_items - 1, deleted))
                }
            }
        }
    }

    pub fn delete(&mut self, selection: &Selection) {
        match selection {
            Selection::Single(index) => self.delete_single(*index),
            Selection::Multiple(items) => self.delete_multiple(items),
        }
    }

    /// Add a number of rows at a specific index
    pub fn add_rows_at(&mut self, rows: Vec<T>, at: usize) {
        if self.is_empty() {
            self.0 = rows;
        } else {
            if at > self.len() {
                self.append(&mut rows.clone());
            } else {
                self.splice(at..at, rows);
            }
        };
    }
}

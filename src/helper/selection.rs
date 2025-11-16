use crate::action::Selection;

pub struct ModifiableList<T: Clone>(pub Vec<T>);

impl<T: Clone> ModifiableList<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self(items)
    }
    /// Delete a row specified by the provided index
    fn delete_single(&mut self, index: usize) {
        self.0.remove(index);
    }

    /// Delete a range
    fn delete_range(&mut self, start: usize, end: usize) {
        self.0.drain(start..=end);
    }

    /// Delete rows where selected[index] is true
    fn delete_multiple(&mut self, selected: &[bool]) {
        self.0 = self
            .0
            .iter()
            .zip(selected)
            .filter_map(|(item, &flag)| if flag { Some(item.clone()) } else { None })
            .collect();
    }

    /// This function answers the following question:
    /// "If the given selection is deleted, where should the cursor at the given index should go?"
    ///
    /// Returns the new index, or returns None if all items are deleted so that there can be no
    /// valid position for the cursor to settle at.
    pub fn move_item_to(&self, selection: &Selection, idx: usize) -> Option<usize> {
        let len = self.0.len();
        match selection {
            Selection::Single(index) => {
                if len == 1 {
                    None
                } else if idx == *index {
                    if idx == 0 {
                        Some(0)
                    } else {
                        Some(idx - 1)
                    }
                } else {
                    if idx > *index {
                        Some(idx - 1)
                    } else {
                        Some(idx)
                    }
                }
            }
            Selection::Range(start, end) => {
                if *start == 0 && *end == (len - 1) {
                    None
                } else if idx < *start {
                    Some(idx)
                } else if idx > *end {
                    Some(idx - (end - start + 1))
                } else {
                    Some(start - 1)
                }
            }
            Selection::Multiple(items) => {
                let mut safe_items = 0;
                for i in 0..idx {
                    if !items[i] {
                        safe_items += 1;
                    }
                }
                if safe_items == 0 {
                    for i in idx..len {
                        if !items[i] {
                            return Some(0);
                        }
                    }
                    None
                } else {
                    Some(safe_items - 1)
                }
            }
        }
    }

    /// This function answers the following question:
    /// "If the given selection is deleted, where should the item at the given index go?"
    ///
    /// Returns the new index, or returns None if the item at the index is deleted.
    // pub fn move_item_by(&self, selection: &Selection, idx: usize) -> Option<usize> {
    //     match selection {
    //         Selection::Single(index) => {
    //             if idx == *index {
    //                 None
    //             } else {
    //                 if idx > *index {
    //                     Some(idx - 1)
    //                 } else {
    //                     Some(idx)
    //                 }
    //             }
    //         }
    //         Selection::Range(start, end) => {
    //             if idx < *start {
    //                 Some(idx)
    //             } else if idx > *end {
    //                 Some(idx - (end - start + 1))
    //             } else {
    //                 None
    //             }
    //         }
    //         Selection::Multiple(items) => {
    //             if items[idx] {
    //                 None
    //             } else {
    //                 let mut count = 0;
    //                 for i in 0..idx {
    //                     if items[i] {
    //                         count += 1;
    //                     }
    //                 }
    //                 Some(idx - count)
    //             }
    //         }
    //     }
    // }

    pub fn delete(&mut self, selection: &Selection) {
        match selection {
            Selection::Single(index) => self.delete_single(*index),
            Selection::Range(start, end) => self.delete_range(*start, *end),
            Selection::Multiple(items) => self.delete_multiple(items),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Add a number of rows at a specific index
    pub fn add_rows_at(&mut self, rows: Vec<T>, at: usize) {
        if self.0.is_empty() {
            self.0 = rows;
        } else {
            if at > self.0.len() {
                self.0.append(&mut rows.clone());
            } else {
                self.0.splice(at..at, rows);
            }
        };
    }
}

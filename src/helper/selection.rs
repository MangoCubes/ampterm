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

    /// Set all the rows with a new set of rows, then signal the table that there have been
    /// additional elements at the specified index
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

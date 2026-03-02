use crate::components::traits::renderable::Renderable;

/// Any component that implements this trait implies that the component can use the given filter to
/// hide unwanted contents
pub trait HandleFilter: Renderable {
    /// Applies filter, hiding anything that do not fit the criteria
    fn set_filter(&mut self, filter: String) -> Action {
        let mut count = 0;
        let visibility: Vec<bool> = self
            .playlist
            .entry
            .iter()
            .map(|i| {
                let a = i.title.to_lowercase().contains(&filter.to_lowercase());
                if a {
                    count += 1;
                }
                a
            })
            .collect();
        self.filter = Some((count, filter));
        self.state = State::Nothing;
        self.table.set_visibility(&visibility);
        self.table.bump_cursor_pos();
        self.bar.update_max(count as u32);
        Action::ChangeMode(Mode::Normal)
    }
    /// Resets the filter, revealing all elements that were hidden
    fn reset_filter(&mut self) -> Action {
        self.state = State::Nothing;
        self.filter = None;
        self.table.reset_visibility();
        self.table.bump_cursor_pos();
        self.bar.update_max(self.playlist.entry.len() as u32);
        Action::ChangeMode(Mode::Normal)
    }
    fn exit_filter(&mut self) -> Action {
        self.state = State::Nothing;
        self.table.bump_cursor_pos();
        Action::ChangeMode(Mode::Normal)
    }
}

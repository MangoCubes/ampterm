use crate::{
    action::{
        useraction::{Common, Normal, UserAction, Visual},
        Action,
    },
    app::Mode,
    components::{
        home::mainscreen::playlistqueue::PlaylistQueue,
        lib::visualtable::{TempSelection, VisualTable},
        traits::{component::Component, focusable::Focusable},
    },
    osclient::response::getplaylist::{FullPlaylist, Media},
    playerworker::player::{QueueLocation, ToPlayerWorker},
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{getplaylist::GetPlaylistParams, ToQueryWorker},
    },
};
use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Row, Table, TableState},
    Frame,
};

pub struct Loaded {
    name: String,
    playlist: FullPlaylist,
    enabled: bool,
    table: VisualTable,
}

impl Loaded {
    fn add_temp_items_to_queue(
        &mut self,
        selection: TempSelection,
        playpos: QueueLocation,
    ) -> Option<Action> {
        self.table.disable_visual_discard();
        if selection.is_select {
            let slice = &self.playlist.entry[selection.start..=selection.end];
            Some(Action::ToPlayerWorker(ToPlayerWorker::AddToQueue {
                pos: playpos,
                music: slice.to_vec(),
            }))
        } else {
            None
        }
    }
    fn add_selection_to_queue(&mut self, playpos: QueueLocation) -> Option<Action> {
        let selection = self.table.get_current_selection();
        let items: Vec<Media> = selection
            .into_iter()
            .enumerate()
            .filter(|(_, selected)| **selected)
            .filter_map(|(idx, _)| self.playlist.entry.get(idx))
            .map(|m| m.clone())
            .collect();
        if items.len() == 0 {
            let cur_pos = self
                .table
                .get_current()
                .expect("Failed to get current cursor position!");
            Some(Action::ToPlayerWorker(ToPlayerWorker::AddToQueue {
                pos: playpos,
                music: vec![self.playlist.entry[cur_pos].clone()],
            }))
        } else {
            self.table.reset();
            Some(Action::ToPlayerWorker(ToPlayerWorker::AddToQueue {
                pos: playpos,
                music: items,
            }))
        }
    }
    pub fn gen_rows(items: &Vec<Media>) -> Vec<Row<'static>> {
        items
            .iter()
            .map(|item| Row::new(vec![item.title.clone(), item.get_fav_marker()]))
            .collect()
    }

    pub fn new(name: String, list: FullPlaylist, enabled: bool) -> Self {
        fn table_proc(table: Table<'static>) -> Table<'static> {
            table
                .highlight_symbol(">")
                .row_highlight_style(Style::new().reversed())
        }
        let rows = Self::gen_rows(&list.entry);
        let table = VisualTable::new(
            rows,
            [Constraint::Min(0), Constraint::Max(1)].to_vec(),
            table_proc,
        );
        let mut tablestate = TableState::default();
        tablestate.select(Some(0));

        Self {
            name,
            enabled,
            playlist: list,
            table,
        }
    }
}

impl Component for Loaded {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let title = if let Some(pos) = self.table.get_current() {
            let len = self.playlist.entry.len();
            format!(
                "{} ({}/{})",
                self.name,
                if pos == usize::MAX || pos >= len {
                    len
                } else {
                    pos + 1
                },
                len
            )
        } else {
            format!("{} ({})", self.name, self.playlist.entry.len())
        };
        let border = PlaylistQueue::gen_block(self.enabled, title);
        let inner = border.inner(area);
        frame.render_widget(border, area);
        self.table.draw(frame, inner)
    }
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::User(ua) = action {
            match ua {
                UserAction::Common(a) => match a {
                    Common::Refresh => Ok(Some(Action::ToQueryWorker(ToQueryWorker::new(
                        HighLevelQuery::SelectPlaylist(GetPlaylistParams {
                            name: self.name.to_string(),
                            id: self.playlist.id.clone(),
                        }),
                    )))),
                    _ => self.table.update(Action::User(UserAction::Common(a))),
                },
                UserAction::Normal(a) => match a {
                    Normal::Add(queue_location) => Ok(self.add_selection_to_queue(queue_location)),
                    _ => self.table.update(Action::User(UserAction::Normal(a))),
                },
                UserAction::Visual(a) => match a {
                    Visual::Add(queue_location) => {
                        if let Some(range) = self.table.get_temp_range() {
                            let temp_action = self.add_temp_items_to_queue(range, queue_location);
                            if let Some(a) = temp_action {
                                Ok(Some(Action::Multiple(vec![
                                    a,
                                    Action::ChangeMode(Mode::Normal),
                                ])))
                            } else {
                                Ok(Some(Action::ChangeMode(Mode::Normal)))
                            }
                        } else {
                            Ok(None)
                        }
                    }
                    _ => self.table.update(Action::User(UserAction::Visual(a))),
                },
            }
        } else {
            self.table.update(action)
        }
    }
}

impl Focusable for Loaded {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            if !self.enabled {
                self.table.disable_visual_discard();
            }
        };
    }
}

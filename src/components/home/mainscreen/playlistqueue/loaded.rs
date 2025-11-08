use crate::{
    action::{
        useraction::{Common, UserAction},
        Action,
    },
    app::Mode,
    components::{
        home::mainscreen::playlistqueue::PlaylistQueue,
        lib::visualtable::{TempSelection, VisualTable},
        traits::{focusable::Focusable, fullcomp::FullComp, renderable::Renderable},
    },
    osclient::response::getplaylist::{FullPlaylist, Media},
    playerworker::player::{QueueLocation, ToPlayerWorker},
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{
            getplaylist::{GetPlaylistParams, MediaID},
            ToQueryWorker,
        },
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
    /// Adds temporarily selected items into the queue. Also quits selection mode at the same time.
    /// Does not work if the current mode is not select mode. Takes priority over
    /// [`Loaded::add_selection_to_queue`].
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

    /// Adds selected items into the queue, resetting the current selection. If temporary selection
    /// is present, this action is NOT taken in favour of [`Loaded::add_temp_items_to_queue`].
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

    /// Generate rows so that they can be used by the table component
    pub fn gen_rows(items: &Vec<Media>) -> Vec<Row<'static>> {
        items
            .iter()
            .map(|item| {
                Row::new(vec![
                    item.artist.clone().unwrap_or("Unknown".to_string()),
                    item.title.clone(),
                    item.get_fav_marker(),
                ])
            })
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
            [
                Constraint::Ratio(1, 3),
                Constraint::Ratio(2, 3),
                Constraint::Min(1),
            ]
            .to_vec(),
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

    pub fn set_star(&mut self, media: MediaID, star: bool) -> Option<Action> {
        let updated = self
            .playlist
            .entry
            .clone()
            .into_iter()
            .map(|mut m| {
                if m.id == media {
                    m.starred = if star {
                        Some("Starred".to_string())
                    } else {
                        None
                    };
                }
                m
            })
            .collect();
        self.playlist.entry = updated;
        let rows = Self::gen_rows(&self.playlist.entry);
        self.table.set_rows(rows);
        None
    }
}

impl Renderable for Loaded {
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
}

impl FullComp for Loaded {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::User(ua) = action {
            match ua {
                UserAction::Common(a) => match a {
                    Common::ToggleStar => {
                        if let Some(range) = self.table.get_temp_range() {
                            self.table.disable_visual_discard();
                            if range.is_select {
                                let slice = &self.playlist.entry[range.start..=range.end];
                                let mut actions: Vec<Action> = slice
                                    .into_iter()
                                    .map(|m| {
                                        Action::ToQueryWorker(ToQueryWorker::new(
                                            HighLevelQuery::SetStar {
                                                media: m.id.clone(),
                                                star: m.starred == None,
                                            },
                                        ))
                                    })
                                    .collect();
                                actions.push(Action::ChangeMode(Mode::Normal));

                                Ok(Some(Action::Multiple(actions)))
                            } else {
                                Ok(Some(Action::ChangeMode(Mode::Normal)))
                            }
                        } else {
                            let selection = self.table.get_current_selection();
                            let targets: Vec<Action> = selection
                                .into_iter()
                                .enumerate()
                                .filter_map(|(idx, include)| {
                                    if *include {
                                        self.playlist.entry.get(idx)
                                    } else {
                                        None
                                    }
                                })
                                .map(|m| {
                                    Action::ToQueryWorker(ToQueryWorker::new(
                                        HighLevelQuery::SetStar {
                                            media: m.id.clone(),
                                            star: m.starred == None,
                                        },
                                    ))
                                })
                                .collect();
                            if targets.len() == 0 {
                                let Some(idx) = self.table.get_current() else {
                                    return Ok(None);
                                };
                                let Some(media) = self.playlist.entry.get(idx) else {
                                    return Ok(None);
                                };
                                Ok(Some(Action::ToQueryWorker(ToQueryWorker::new(
                                    HighLevelQuery::SetStar {
                                        media: media.id.clone(),
                                        star: media.starred.is_none(),
                                    },
                                ))))
                            } else {
                                Ok(Some(Action::Multiple(targets)))
                            }
                        }
                    }
                    Common::Add(pos) => {
                        if let Some(range) = self.table.get_temp_range() {
                            let temp_action = self.add_temp_items_to_queue(range, pos);
                            if let Some(a) = temp_action {
                                Ok(Some(Action::Multiple(vec![
                                    a,
                                    Action::ChangeMode(Mode::Normal),
                                ])))
                            } else {
                                Ok(Some(Action::ChangeMode(Mode::Normal)))
                            }
                        } else {
                            Ok(self.add_selection_to_queue(pos))
                        }
                    }
                    Common::Refresh => Ok(Some(Action::ToQueryWorker(ToQueryWorker::new(
                        HighLevelQuery::SelectPlaylist(GetPlaylistParams {
                            name: self.name.to_string(),
                            id: self.playlist.id.clone(),
                        }),
                    )))),
                    _ => self.table.update(Action::User(UserAction::Common(a))),
                },
                UserAction::Visual(_) | UserAction::Normal(_) => {
                    self.table.update(Action::User(ua))
                }
                _ => Ok(None),
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

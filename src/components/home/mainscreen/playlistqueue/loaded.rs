use crate::{
    action::{
        useraction::{Common, Normal, UserAction, Visual},
        Action,
    },
    app::Mode,
    components::{
        home::mainscreen::playlistqueue::PlaylistQueue,
        lib::visualstate::VisualState,
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
    visual: VisualState,
    enabled: bool,
    table: Table<'static>,
    tablestate: TableState,
}

impl Loaded {
    fn add_temp_items_to_queue(
        &mut self,
        selection: (usize, usize, bool),
        playpos: QueueLocation,
    ) -> Option<Action> {
        let (start, end, is_select) = selection;
        self.visual.disable_visual_discard();
        if is_select {
            let slice = &self.playlist.entry[start..=end];
            Some(Action::ToPlayerWorker(ToPlayerWorker::AddToQueue {
                pos: playpos,
                music: slice.to_vec(),
            }))
        } else {
            None
        }
    }
    fn add_selection_to_queue(&mut self, playpos: QueueLocation) -> Option<Action> {
        let selection = self.visual.get_current_selection();
        let items: Vec<Media> = selection
            .into_iter()
            .enumerate()
            .filter(|(_, selected)| **selected)
            .filter_map(|(idx, _)| self.playlist.entry.get(idx))
            .map(|m| m.clone())
            .collect();
        if items.len() == 0 {
            let cur_pos = self
                .tablestate
                .selected()
                .expect("Failed to get current cursor location.");
            Some(Action::ToPlayerWorker(ToPlayerWorker::AddToQueue {
                pos: playpos,
                music: vec![self.playlist.entry[cur_pos].clone()],
            }))
        } else {
            self.visual.reset();
            Some(Action::ToPlayerWorker(ToPlayerWorker::AddToQueue {
                pos: playpos,
                music: items,
            }))
        }
    }
    pub fn regen_table(&self) -> Table<'static> {
        let cur_pos = self
            .tablestate
            .selected()
            .expect("Failed to get current cursor location.");

        Self::gen_table(
            &self.playlist.entry,
            self.visual.get_temp_selection(cur_pos),
            self.visual.get_current_selection(),
        )
    }
    pub fn gen_table(
        items: &Vec<Media>,
        temp: Option<(usize, usize, bool)>,
        sel: &[bool],
    ) -> Table<'static> {
        let iter = items.iter().enumerate();
        let rows: Vec<Row> = match temp {
            Some((a, b, _)) => iter
                .map(|(i, item)| {
                    let mut row = Row::new(vec![item.title.clone(), item.get_fav_marker()]);
                    row = if i <= b && i >= a {
                        row.reversed()
                    } else {
                        row
                    };
                    if sel[i] {
                        row.green()
                    } else {
                        row
                    }
                })
                .collect(),
            None => iter
                .map(|(i, item)| {
                    let row = Row::new(vec![item.title.clone(), item.get_fav_marker()]);
                    if sel[i] {
                        row.green()
                    } else {
                        row
                    }
                })
                .collect(),
        };
        Table::new(rows, [Constraint::Min(0), Constraint::Max(1)].to_vec())
            .highlight_symbol(">")
            .row_highlight_style(Style::new().reversed())
    }

    pub fn new(name: String, list: FullPlaylist, enabled: bool) -> Self {
        let len = list.entry.len();
        let table = Self::gen_table(&list.entry, None, &vec![false; len]);
        let mut tablestate = TableState::default();
        tablestate.select(Some(0));

        Self {
            name,
            visual: VisualState::new(len),
            enabled,
            playlist: list,
            table,
            tablestate,
        }
    }
}

impl Component for Loaded {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let title = if let Some(pos) = self.tablestate.selected() {
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
        frame.render_stateful_widget(&self.table, inner, &mut self.tablestate);
        Ok(())
    }
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::User(ua) => {
                let cur_pos = self
                    .tablestate
                    .selected()
                    .expect("Failed to get current cursor location.");

                let action = match ua {
                    UserAction::Common(local) => match local {
                        Common::Up => {
                            self.tablestate.select_previous();
                            Ok(None)
                        }
                        Common::Down => {
                            self.tablestate.select_next();
                            Ok(None)
                        }
                        Common::Top => {
                            self.tablestate.select_first();
                            Ok(None)
                        }
                        Common::Bottom => {
                            self.tablestate.select_last();
                            Ok(None)
                        }
                        Common::Refresh => Ok(Some(Action::ToQueryWorker(ToQueryWorker::new(
                            HighLevelQuery::SelectPlaylist(GetPlaylistParams {
                                name: self.name.to_string(),
                                id: self.playlist.id.clone(),
                            }),
                        )))),
                        _ => Ok(None),
                    },

                    UserAction::Normal(normal) => match normal {
                        Normal::SelectMode => {
                            self.visual.enable_visual(cur_pos, false);
                            Ok(Some(Action::ChangeMode(Mode::Visual)))
                        }
                        Normal::DeselectMode => {
                            self.visual.enable_visual(cur_pos, true);
                            Ok(Some(Action::ChangeMode(Mode::Visual)))
                        }
                        Normal::Add(queue_location) => {
                            Ok(self.add_selection_to_queue(queue_location))
                        }
                        _ => Ok(None),
                    },
                    UserAction::Visual(visual) => match visual {
                        Visual::ExitSave => {
                            self.visual.disable_visual(cur_pos);
                            Ok(Some(Action::ChangeMode(Mode::Normal)))
                        }
                        Visual::ExitDiscard => {
                            self.visual.disable_visual_discard();
                            Ok(Some(Action::ChangeMode(Mode::Normal)))
                        }
                        Visual::Add(queue_location) => {
                            if let Some(items) = self.visual.get_temp_selection(cur_pos) {
                                let temp_action =
                                    self.add_temp_items_to_queue(items, queue_location);

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
                    },
                    _ => Ok(None),
                };
                self.table = self.regen_table();
                action
            }
            _ => Ok(None),
        }
    }
}

impl Focusable for Loaded {
    fn set_enabled(&mut self, enable: bool) {
        if self.enabled != enable {
            self.enabled = enable;
            if !self.enabled {
                self.visual.disable_visual_discard();
            }
        };
    }
}

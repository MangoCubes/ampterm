mod bpmtoy;
mod nowplaying;
pub mod playlistlist;
mod playlistqueue;
mod playqueue;
mod tasks;

use crate::{
    action::{
        action::Mode,
        useraction::{Global, Normal, UserAction},
    },
    compid::CompID,
    components::{
        home::mainscreen::{bpmtoy::BPMToy, tasks::Tasks},
        traits::{
            focusable::Focusable,
            handleaction::{HandleAction, HandleActionSimple},
            handlekeyseq::{HandleKeySeq, KeySeqResult, PassKeySeq},
            handleraw::HandleRaw,
            ontick::OnTick,
            renderable::Renderable,
        },
    },
    config::Config,
    queryworker::{highlevelquery::HighLevelQuery, query::ToQueryWorker},
};
use crossterm::event::{KeyEvent, KeyModifiers};
use nowplaying::NowPlaying;
use playlistlist::PlaylistList;
use playlistqueue::PlaylistQueue;
use playqueue::PlayQueue;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    widgets::{Paragraph, Wrap},
    Frame,
};

#[derive(PartialEq, Clone)]
enum LastSelected {
    PlaylistList,
    Playlist,
    PlayQueue,
}

#[derive(PartialEq)]
enum CurrentlySelected {
    PlaylistList,
    Playlist,
    PlayQueue,
    NowPlaying(LastSelected),
}

pub struct MainScreen {
    state: CurrentlySelected,
    pl_list: PlaylistList,
    pl_queue: PlaylistQueue,
    now_playing: NowPlaying,
    tasks: Tasks,
    show_tasks: bool,
    bpmtoy: BPMToy,
    playqueue: PlayQueue,
    message: String,
    key_stack: Vec<String>,
    current_mode: Mode,
}

impl OnTick for MainScreen {
    fn on_tick(&mut self) {
        self.bpmtoy.on_tick();
    }
}

impl PassKeySeq for MainScreen {
    fn handle_key_seq(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult> {
        if self.show_tasks {
            None
        } else {
            match self.state {
                CurrentlySelected::PlaylistList => self.pl_list.handle_key_seq(keyseq),
                CurrentlySelected::Playlist => self.pl_queue.handle_key_seq(keyseq),
                CurrentlySelected::PlayQueue => self.playqueue.handle_key_seq(keyseq),
                CurrentlySelected::NowPlaying(_) => self.now_playing.handle_key_seq(keyseq),
            }
        }
    }
}

impl MainScreen {
    fn propagate_to_focused_component(&mut self, action: Action) -> Option<Action> {
        if self.show_tasks {
            // self.tasks.update(action)
            None
        } else {
            match self.state {
                CurrentlySelected::PlaylistList => self.pl_list.handle_action(action),
                CurrentlySelected::Playlist => self.pl_queue.handle_action(action),
                CurrentlySelected::PlayQueue => self.playqueue.handle_action(action),
                CurrentlySelected::NowPlaying(_) => self.now_playing.handle_action(action),
            }
        }
    }
    pub fn new(config: Config) -> (Self, Action) {
        (
            Self {
                state: CurrentlySelected::PlaylistList,
                current_mode: Mode::Normal,
                pl_list: PlaylistList::new(config.clone(), true),
                pl_queue: PlaylistQueue::new(false),
                playqueue: PlayQueue::new(false, config.clone()),
                now_playing: NowPlaying::new(false, config.clone()),
                tasks: Tasks::new(config.behaviour.show_internal_tasks),
                bpmtoy: BPMToy::new(config),
                message: "You are now logged in.".to_string(),
                key_stack: vec![],
                show_tasks: false,
            },
            Action::Multiple(vec![
                Action::ToQueryWorker(ToQueryWorker::new(HighLevelQuery::ListPlaylists)),
                Action::ChangeMode(Mode::Normal),
            ]),
        )
    }
    fn update_focus(&mut self) {
        self.pl_list
            .set_enabled(self.state == CurrentlySelected::PlaylistList);
        self.pl_queue
            .set_enabled(self.state == CurrentlySelected::Playlist);
        self.playqueue
            .set_enabled(self.state == CurrentlySelected::PlayQueue);
        self.now_playing
            .set_enabled(matches!(self.state, CurrentlySelected::NowPlaying(_)));
    }
}

impl Renderable for MainScreen {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(10),
            Constraint::Length(1),
        ]);
        let horizontal = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ]);
        let text_layout = Layout::horizontal([
            Constraint::Length(9),
            Constraint::Ratio(3, 4),
            Constraint::Ratio(1, 4),
        ]);
        let bottom_layout =
            Layout::horizontal([Constraint::Percentage(75), Constraint::Percentage(25)]);
        let areas = vertical.split(area);
        let listareas = horizontal.split(areas[1]);
        let text_areas = text_layout.split(areas[3]);
        let bottom_areas = bottom_layout.split(areas[2]);
        self.pl_list.draw(frame, listareas[0]);
        self.pl_queue.draw(frame, listareas[1]);
        self.playqueue.draw(frame, listareas[2]);
        self.now_playing.draw(frame, bottom_areas[0]);
        self.bpmtoy.draw(frame, bottom_areas[1]);

        if self.show_tasks {
            self.tasks.draw(frame, area);
        }

        frame.render_widget(
            Paragraph::new(format!("[{}]", self.current_mode.to_string()))
                .wrap(Wrap { trim: false }),
            text_areas[0],
        );

        frame.render_widget(
            Paragraph::new(format!(
                "Ampterm {} | Tasks: {}",
                env!("CARGO_PKG_VERSION"),
                self.tasks.get_task_count()
            ))
            .wrap(Wrap { trim: false }),
            areas[0],
        );
        frame.render_widget(
            Paragraph::new(self.message.clone()).wrap(Wrap { trim: false }),
            text_areas[1],
        );
        frame.render_widget(
            Paragraph::new(self.key_stack.join(" ")).wrap(Wrap { trim: false }),
            text_areas[2],
        );
    }
}

impl HandleRaw for MainScreen {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        self.key_stack.push(format!(
            "{}{}",
            key.code.to_string(),
            if key.modifiers == KeyModifiers::NONE {
                ""
            } else {
                "+"
            },
        ));
        None
    }
}

impl HandleAction for MainScreen {
    fn handle_action(&mut self, action: Action) -> Option<Action> {
        if matches!(action, Action::User(_)) {
            self.key_stack.drain(..);
        };
        match &action {
            Action::FromPlayerWorker(pw) => match pw {
                FromPlayerWorker::StateChange(_) => {
                    let results: Vec<Action> = [
                        self.now_playing.handle_action(action.clone()),
                        self.playqueue.handle_action(action),
                    ]
                    .into_iter()
                    .filter_map(|a| a)
                    .collect();

                    Some(Action::Multiple(results))
                }
                FromPlayerWorker::Error(msg) | FromPlayerWorker::Message(msg) => {
                    self.message = msg.clone();
                    None
                }
                FromPlayerWorker::Finished => self.playqueue.handle_action(action),
            },
            Action::ChangeMode(m) => {
                self.current_mode = *m;
                None
            }
            Action::User(u) => match u {
                UserAction::Normal(normal) => match normal {
                    Normal::WindowUp | Normal::WindowDown => {
                        self.state = match &self.state {
                            CurrentlySelected::PlaylistList => {
                                CurrentlySelected::NowPlaying(LastSelected::PlaylistList)
                            }
                            CurrentlySelected::Playlist => {
                                CurrentlySelected::NowPlaying(LastSelected::Playlist)
                            }
                            CurrentlySelected::PlayQueue => {
                                CurrentlySelected::NowPlaying(LastSelected::PlayQueue)
                            }
                            CurrentlySelected::NowPlaying(last_selected) => match last_selected {
                                LastSelected::PlaylistList => CurrentlySelected::PlaylistList,
                                LastSelected::Playlist => CurrentlySelected::Playlist,
                                LastSelected::PlayQueue => CurrentlySelected::PlayQueue,
                            },
                        };
                        self.update_focus();
                        None
                    }
                    Normal::WindowLeft => {
                        match &self.state {
                            CurrentlySelected::PlaylistList => {
                                self.state = CurrentlySelected::PlayQueue;
                            }
                            CurrentlySelected::PlayQueue => {
                                self.state = CurrentlySelected::Playlist;
                            }
                            CurrentlySelected::Playlist => {
                                self.state = CurrentlySelected::PlaylistList;
                            }
                            CurrentlySelected::NowPlaying(_) => {}
                        };
                        self.update_focus();
                        None
                    }
                    Normal::WindowRight => {
                        match &self.state {
                            CurrentlySelected::PlaylistList => {
                                self.state = CurrentlySelected::Playlist;
                            }
                            CurrentlySelected::Playlist => {
                                self.state = CurrentlySelected::PlayQueue;
                            }
                            CurrentlySelected::PlayQueue => {
                                self.state = CurrentlySelected::PlaylistList;
                            }
                            CurrentlySelected::NowPlaying(_) => {}
                        };
                        self.update_focus();
                        None
                    }
                    _ => self.propagate_to_focused_component(action),
                },
                UserAction::Global(global) => match global {
                    Global::TapToBPM => {
                        self.bpmtoy.handle_action_simple(action);
                        None
                    }
                    Global::FocusPlaylistList => {
                        self.state = CurrentlySelected::PlaylistList;
                        self.update_focus();
                        None
                    }
                    Global::FocusPlaylistQueue => {
                        self.state = CurrentlySelected::Playlist;
                        self.update_focus();
                        None
                    }
                    Global::FocusPlayQueue => {
                        self.state = CurrentlySelected::PlayQueue;
                        self.update_focus();
                        None
                    }
                    Global::EndKeySeq => None,
                    Global::OpenTasks => {
                        self.show_tasks = true;
                        None
                    }
                    Global::CloseTasks => {
                        self.show_tasks = false;
                        None
                    }
                    Global::ToggleTasks => {
                        self.show_tasks = !self.show_tasks;
                        None
                    }
                    Global::Skip | Global::Previous => self.playqueue.handle_action(action),
                },
                _ => self.propagate_to_focused_component(action),
            },
            Action::ToQueryWorker(req) => {
                let _ = self.tasks.register_task(req.clone());
                let mut results = vec![];

                for dest in &req.dest {
                    let res = match dest {
                        CompID::PlaylistList => self.pl_list.handle_action(action.clone()),
                        CompID::PlaylistQueue => self.pl_queue.handle_action(action.clone()),
                        CompID::PlayQueue => self.playqueue.handle_action(action.clone()),
                        CompID::NowPlaying => self.now_playing.handle_action(action.clone()),
                        CompID::None => None,
                        _ => unreachable!("Action propagated to nonexistent component: {:?}", dest),
                    };
                    if let Some(a) = res {
                        results.push(a);
                    }
                }
                Some(Action::Multiple(results))
            }
            Action::FromQueryWorker(res) => {
                let _ = self.tasks.unregister_task(res);
                let mut results = vec![];

                for dest in &res.dest {
                    let res = match dest {
                        CompID::PlaylistList => self.pl_list.handle_action(action.clone()),
                        CompID::PlaylistQueue => self.pl_queue.handle_action(action.clone()),
                        CompID::PlayQueue => self.playqueue.handle_action(action.clone()),
                        CompID::NowPlaying => self.now_playing.handle_action(action.clone()),
                        CompID::None => None,
                        _ => unreachable!("Action propagated to nonexistent component: {:?}", dest),
                    };
                    if let Some(a) = res {
                        results.push(a);
                    }
                }
                Some(Action::Multiple(results))
            }
            _ => {
                self.bpmtoy.handle_action_simple(action.clone());
                let results: Vec<Action> = [
                    self.pl_list.handle_action(action.clone()),
                    self.pl_queue.handle_action(action.clone()),
                    self.now_playing.handle_action(action.clone()),
                    self.playqueue.handle_action(action),
                    // self.tasks.update(action.clone())?,
                ]
                .into_iter()
                .filter_map(|a| a)
                .collect();

                Some(Action::Multiple(results))
            }
        }
    }
}

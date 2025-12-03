mod bpmtoy;
mod nowplaying;
pub mod playlistlist;
mod playlistqueue;
mod playqueue;
mod tasks;

use crate::{
    action::action::{Action, Mode, QueryAction, TargetedAction},
    compid::CompID,
    components::{
        home::mainscreen::{bpmtoy::BPMToy, tasks::Tasks},
        traits::{
            focusable::Focusable,
            handleaction::HandleAction,
            handlekeyseq::{ComponentKeyHelp, KeySeqResult, PassKeySeq},
            handlemode::HandleMode,
            handlequery::HandleQuery,
            handleraw::HandleRaw,
            ontick::OnTick,
            renderable::Renderable,
        },
    },
    config::Config,
    playerworker::player::FromPlayerWorker,
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
    fn get_help(&self) -> Vec<ComponentKeyHelp> {
        match &self.state {
            CurrentlySelected::PlaylistList => self.pl_list.get_help(),
            CurrentlySelected::Playlist => self.pl_queue.get_help(),
            CurrentlySelected::PlayQueue => self.playqueue.get_help(),
            CurrentlySelected::NowPlaying(_) => self.now_playing.get_help(),
        }
    }
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

impl HandleMode for MainScreen {
    fn handle_mode(&mut self, mode: Mode) {
        self.current_mode = mode;
    }
}

impl MainScreen {
    pub fn new(config: Config) -> (Self, Action) {
        (
            Self {
                state: CurrentlySelected::PlaylistList,
                current_mode: Mode::Normal,
                pl_list: PlaylistList::new(config.clone(), true),
                pl_queue: PlaylistQueue::new(config.clone(), false),
                playqueue: PlayQueue::new(false, config.clone()),
                now_playing: NowPlaying::new(false, config.clone()),
                tasks: Tasks::new(config.behaviour.show_internal_tasks),
                bpmtoy: BPMToy::new(config),
                message: "You are now logged in.".to_string(),
                key_stack: vec![],
                show_tasks: false,
            },
            Action::Multiple(vec![
                Action::Query(QueryAction::ToQueryWorker(ToQueryWorker::new(
                    HighLevelQuery::ListPlaylists,
                ))),
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

impl HandleQuery for MainScreen {
    fn handle_query(&mut self, action: QueryAction) -> Option<Action> {
        match action {
            QueryAction::ToQueryWorker(ref req) => {
                let _ = self.tasks.register_task(req.clone());
                let mut results = vec![];

                for dest in &req.dest {
                    let res = match dest {
                        CompID::PlaylistList => self.pl_list.handle_query(action.clone()),
                        CompID::PlaylistQueue => self.pl_queue.handle_query(action.clone()),
                        CompID::PlayQueue => self.playqueue.handle_query(action.clone()),
                        CompID::NowPlaying => self.now_playing.handle_query(action.clone()),
                        CompID::None => None,
                        _ => unreachable!("Action propagated to nonexistent component: {:?}", dest),
                    };
                    if let Some(a) = res {
                        results.push(a);
                    }
                }
                Some(Action::Multiple(results))
            }
            QueryAction::FromQueryWorker(ref res) => {
                let _ = self.tasks.unregister_task(&res);
                let mut results = vec![];

                for dest in &res.dest {
                    let res = match dest {
                        CompID::PlaylistList => self.pl_list.handle_query(action.clone()),
                        CompID::PlaylistQueue => self.pl_queue.handle_query(action.clone()),
                        CompID::PlayQueue => self.playqueue.handle_query(action.clone()),
                        CompID::NowPlaying => self.now_playing.handle_query(action.clone()),
                        CompID::None => None,
                        _ => unreachable!("Action propagated to nonexistent component: {:?}", dest),
                    };
                    if let Some(a) = res {
                        results.push(a);
                    }
                }
                Some(Action::Multiple(results))
            }
            QueryAction::FromPlayerWorker(ref pw) => match pw {
                FromPlayerWorker::StateChange(_) => {
                    let results: Vec<Action> = [
                        self.now_playing.handle_query(action.clone()),
                        self.playqueue.handle_query(action),
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
                FromPlayerWorker::Finished => self.playqueue.handle_query(action),
            },
            _ => None,
        }
    }
}

impl HandleAction for MainScreen {
    fn handle_action(&mut self, action: TargetedAction) -> Option<Action> {
        match action {
            TargetedAction::EndKeySeq => None,
            TargetedAction::Queue(_) | TargetedAction::Skip | TargetedAction::Previous => {
                self.playqueue.handle_action(action)
            }
            TargetedAction::WindowUp | TargetedAction::WindowDown => {
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
            TargetedAction::WindowLeft => {
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
            TargetedAction::WindowRight => {
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
            TargetedAction::TapToBPM => self.bpmtoy.handle_action(action),
            TargetedAction::FocusPlaylistList => {
                self.state = CurrentlySelected::PlaylistList;
                self.update_focus();
                None
            }
            TargetedAction::FocusPlaylistQueue => {
                self.state = CurrentlySelected::Playlist;
                self.update_focus();
                None
            }
            TargetedAction::FocusPlayQueue => {
                self.state = CurrentlySelected::PlayQueue;
                self.update_focus();
                None
            }
            TargetedAction::OpenTasks => {
                self.show_tasks = true;
                None
            }
            TargetedAction::CloseTasks => {
                self.show_tasks = false;
                None
            }
            TargetedAction::ToggleTasks => {
                self.show_tasks = !self.show_tasks;
                None
            }
            _ => None,
        }
    }
}

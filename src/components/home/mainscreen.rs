mod bpmtoy;
mod help;
mod nowplaying;
pub mod playlistlist;
mod playlistqueue;
mod playqueue;
mod tasks;

use crate::{
    action::action::{Action, Mode, TargetedAction},
    compid::CompID,
    components::{
        home::mainscreen::{bpmtoy::BPMToy, help::Help, tasks::Tasks},
        traits::{
            focusable::Focusable,
            handleaction::HandleAction,
            handlekeyseq::{ComponentKeyHelp, HandleKeySeq, KeySeqResult, PassKeySeq},
            handlemode::HandleMode,
            handleplayer::HandlePlayer,
            handlequery::HandleQuery,
            handleraw::HandleRaw,
            ontick::OnTick,
            renderable::Renderable,
        },
    },
    config::{keyparser::KeyParser, Config},
    playerworker::player::FromPlayerWorker,
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{QueryStatus, ToQueryWorker},
    },
};
use crossterm::event::KeyEvent;
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
    PlaylistQueue,
    PlayQueue,
    NowPlaying(LastSelected),
}

enum Popup {
    None,
    Tasks,
    Help,
}

pub struct MainScreen {
    state: CurrentlySelected,
    pl_list: PlaylistList,
    pl_queue: PlaylistQueue,
    now_playing: NowPlaying,
    tasks: Tasks,
    popup: Popup,
    bpmtoy: Option<BPMToy>,
    playqueue: PlayQueue,
    message: String,
    key_stack: Vec<String>,
    current_mode: Mode,
    help: Help,
}

impl OnTick for MainScreen {
    fn on_tick(&mut self) {
        if let Some(t) = &mut self.bpmtoy {
            t.on_tick();
        }
    }
}

impl HandleMode for MainScreen {
    fn handle_mode(&mut self, mode: Mode) {
        self.current_mode = mode;
    }
}

// For every function that returns Option<Action>, the following code must be added to ensure all
// tasks are correctly tracked by the task viewer.
//
// if let Some(Action::ToQuery(ToQueryWorker {
//     dest: _,
//     ticket,
//     query,
// })) = &action
// {
//     self.tasks.register_task(ticket, query);
// }
// action

impl HandleRaw for MainScreen {
    fn handle_raw(&mut self, key: KeyEvent) -> Option<Action> {
        let action = match &mut self.state {
            CurrentlySelected::PlaylistQueue => self.pl_queue.handle_raw(key),
            _ => None,
        };
        self.track_task(action)
    }
}

impl PassKeySeq for MainScreen {
    fn get_help(&self) -> Vec<ComponentKeyHelp> {
        match &self.state {
            CurrentlySelected::PlaylistList => self.pl_list.get_help(),
            CurrentlySelected::PlaylistQueue => self.pl_queue.get_help(),
            CurrentlySelected::PlayQueue => self.playqueue.get_help(),
            CurrentlySelected::NowPlaying(_) => self.now_playing.get_help(),
        }
    }
    fn handle_key_seq(&mut self, keyseq: &Vec<KeyEvent>) -> Option<KeySeqResult> {
        if keyseq.len() > 1 {
            if self.key_stack.len() == 0 {
                self.key_stack = keyseq.iter().map(KeyParser::key_event_to_string).collect();
            } else if let Some(k) = keyseq.last() {
                self.key_stack.push(KeyParser::key_event_to_string(k));
            };
        }
        let res = match self.popup {
            Popup::None => match &self.state {
                CurrentlySelected::PlaylistList => self.pl_list.handle_key_seq(keyseq),
                CurrentlySelected::PlaylistQueue => self.pl_queue.handle_key_seq(keyseq),
                CurrentlySelected::PlayQueue => self.playqueue.handle_key_seq(keyseq),
                CurrentlySelected::NowPlaying(_) => self.now_playing.handle_key_seq(keyseq),
            },
            Popup::Tasks => None,
            Popup::Help => self.help.handle_key_seq(keyseq),
        };
        if matches!(res, Some(_)) {
            self.key_stack.drain(..);
        };
        if let Some(KeySeqResult::ActionNeeded(Action::ToQuery(ToQueryWorker {
            dest: _,
            ticket,
            query,
        }))) = &res
        {
            self.tasks.register_task(ticket, query);
        }
        res
    }
}

impl MainScreen {
    fn track_task(&mut self, action: Option<Action>) -> Option<Action> {
        if let Some(Action::ToQuery(ToQueryWorker {
            dest: _,
            ticket,
            query,
        })) = &action
        {
            self.tasks.register_task(ticket, query);
        };
        action
    }
    fn show_help(&mut self) {
        self.help.display(self.get_help());
        self.popup = Popup::Help;
    }
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
                bpmtoy: if config.features.bpmtoy.enable {
                    Some(BPMToy::new(config.clone()))
                } else {
                    None
                },
                message: "You are now logged in.".to_string(),
                key_stack: vec![],
                popup: Popup::None,
                help: Help::new(config),
            },
            Action::Multiple(vec![
                Action::ToQuery(ToQueryWorker::new(HighLevelQuery::ListPlaylists)),
                Action::ChangeMode(Mode::Normal),
            ]),
        )
    }
    fn update_focus(&mut self) {
        self.pl_list
            .set_enabled(self.state == CurrentlySelected::PlaylistList);
        self.pl_queue
            .set_enabled(self.state == CurrentlySelected::PlaylistQueue);
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
        let areas = vertical.split(area);
        let listareas = horizontal.split(areas[1]);
        let text_areas = text_layout.split(areas[3]);
        self.pl_list.draw(frame, listareas[0]);
        self.pl_queue.draw(frame, listareas[1]);
        self.playqueue.draw(frame, listareas[2]);

        if let Some(toy) = &mut self.bpmtoy {
            let bottom_layout =
                Layout::horizontal([Constraint::Percentage(75), Constraint::Percentage(25)]);
            let bottom_areas = bottom_layout.split(areas[2]);
            self.now_playing.draw(frame, bottom_areas[0]);
            toy.draw(frame, bottom_areas[1]);
        } else {
            self.now_playing.draw(frame, areas[2]);
        }

        match self.popup {
            Popup::None => {}
            Popup::Tasks => self.tasks.draw(frame, area),
            Popup::Help => self.help.draw(frame, area),
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

impl HandlePlayer for MainScreen {
    fn handle_player(&mut self, pw: FromPlayerWorker) -> Option<Action> {
        let a = self.now_playing.handle_player(pw.clone());
        let now_playing = self.track_task(a);
        let b = self.playqueue.handle_player(pw);
        let playqueue = self.track_task(b);
        if let Some(a) = now_playing {
            if let Some(b) = playqueue {
                Some(Action::Multiple(vec![a, b]))
            } else {
                Some(a)
            }
        } else {
            playqueue
        }
    }
}

impl HandleQuery for MainScreen {
    fn handle_query(&mut self, dest: CompID, ticket: usize, res: QueryStatus) -> Option<Action> {
        self.tasks.update_task(&ticket, &res);
        let action = match dest {
            CompID::PlaylistList => self.pl_list.handle_query(dest, ticket, res),
            CompID::PlaylistQueue => self.pl_queue.handle_query(dest, ticket, res),
            CompID::NowPlaying | CompID::Lyrics | CompID::ImageComp => {
                self.now_playing.handle_query(dest, ticket, res)
            }
            CompID::PlayQueue => self.playqueue.handle_query(dest, ticket, res),
            _ => unreachable!(),
        };
        self.track_task(action)
    }
}

impl HandleAction for MainScreen {
    fn handle_action(&mut self, action: TargetedAction) -> Option<Action> {
        self.key_stack.drain(..);
        let res = match action {
            TargetedAction::ToggleHelp => {
                if matches!(self.popup, Popup::Help) {
                    self.popup = Popup::None;
                } else {
                    self.show_help();
                };
                None
            }
            TargetedAction::CloseHelp => {
                self.popup = Popup::None;
                None
            }
            TargetedAction::OpenHelp => {
                self.show_help();
                None
            }
            TargetedAction::Queue(_) | TargetedAction::Skip | TargetedAction::Previous => {
                self.playqueue.handle_action(action)
            }
            TargetedAction::WindowUp | TargetedAction::WindowDown => {
                self.state = match &self.state {
                    CurrentlySelected::PlaylistList => {
                        CurrentlySelected::NowPlaying(LastSelected::PlaylistList)
                    }
                    CurrentlySelected::PlaylistQueue => {
                        CurrentlySelected::NowPlaying(LastSelected::Playlist)
                    }
                    CurrentlySelected::PlayQueue => {
                        CurrentlySelected::NowPlaying(LastSelected::PlayQueue)
                    }
                    CurrentlySelected::NowPlaying(last_selected) => match last_selected {
                        LastSelected::PlaylistList => CurrentlySelected::PlaylistList,
                        LastSelected::Playlist => CurrentlySelected::PlaylistQueue,
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
                        self.state = CurrentlySelected::PlaylistQueue;
                    }
                    CurrentlySelected::PlaylistQueue => {
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
                        self.state = CurrentlySelected::PlaylistQueue;
                    }
                    CurrentlySelected::PlaylistQueue => {
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
            TargetedAction::TapToBPM => {
                if let Some(t) = &mut self.bpmtoy {
                    t.handle_action(action)
                } else {
                    None
                }
            }
            TargetedAction::FocusPlaylistList => {
                self.state = CurrentlySelected::PlaylistList;
                self.update_focus();
                None
            }
            TargetedAction::FocusPlaylistQueue => {
                self.state = CurrentlySelected::PlaylistQueue;
                self.update_focus();
                None
            }
            TargetedAction::FocusPlayQueue => {
                self.state = CurrentlySelected::PlayQueue;
                self.update_focus();
                None
            }
            TargetedAction::OpenTasks => {
                self.popup = Popup::Tasks;
                None
            }
            TargetedAction::CloseTasks => {
                self.popup = Popup::None;
                None
            }
            TargetedAction::ToggleTasks => {
                match self.popup {
                    Popup::Tasks => self.popup = Popup::None,
                    _ => self.popup = Popup::Tasks,
                };
                None
            }
            _ => None,
        };
        self.track_task(res)
    }
}

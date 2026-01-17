mod bpmtoy;
mod help;
mod mediainfo;
mod nowplaying;
mod playlistinfo;
pub mod playlistlist;
mod playlistqueue;
mod playqueue;
mod selectplaylistpopup;
mod tasks;

use crate::{
    action::action::{Action, Mode, TargetedAction},
    compid::CompID,
    components::{
        home::mainscreen::{
            bpmtoy::BPMToy, help::Help, mediainfo::MediaInfo, playlistinfo::PlaylistInfo,
            selectplaylistpopup::SelectPlaylistPopup, tasks::Tasks,
        },
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
        query::{QueryStatus, ResponseType, ToQueryWorker},
    },
};
use crossterm::event::KeyEvent;
use nowplaying::NowPlaying;
use playlistlist::PlaylistList;
use playlistqueue::PlaylistQueue;
use playqueue::PlayQueue;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
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
    MediaInfo(MediaInfo),
    PlaylistInfo(PlaylistInfo),
    SelectPlaylist(SelectPlaylistPopup),
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
    message: (bool, String),
    key_stack: Vec<String>,
    current_mode: Mode,
    help: Help,
    config: Config,
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
        match &mut self.state {
            CurrentlySelected::PlaylistQueue => self.pl_queue.handle_raw(key),
            _ => None,
        }
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
        let res = match &mut self.popup {
            Popup::None => match &self.state {
                CurrentlySelected::PlaylistList => self.pl_list.handle_key_seq(keyseq),
                CurrentlySelected::PlaylistQueue => self.pl_queue.handle_key_seq(keyseq),
                CurrentlySelected::PlayQueue => self.playqueue.handle_key_seq(keyseq),
                CurrentlySelected::NowPlaying(_) => self.now_playing.handle_key_seq(keyseq),
            },
            Popup::Tasks => None,
            Popup::Help => self.help.handle_key_seq(keyseq),
            Popup::MediaInfo(comp) => comp.handle_key_seq(keyseq),
            Popup::PlaylistInfo(comp) => comp.handle_key_seq(keyseq),
            Popup::SelectPlaylist(comp) => comp.handle_key_seq(keyseq),
        };
        if matches!(res, Some(_)) {
            self.key_stack.drain(..);
        };
        res
    }
}

impl MainScreen {
    fn show_help(&mut self) {
        self.help.display(self.get_help());
        self.popup = Popup::Help;
    }
    pub fn new(config: Config) -> (Self, Action) {
        (
            Self {
                bpmtoy: if config.features.bpmtoy.enable.clone() {
                    Some(BPMToy::new(config.clone()))
                } else {
                    None
                },
                tasks: Tasks::new(config.behaviour.show_internal_tasks.clone()),
                pl_list: PlaylistList::new(config.clone(), true),
                pl_queue: PlaylistQueue::new(config.clone(), false),
                playqueue: PlayQueue::new(false, config.clone()),
                now_playing: NowPlaying::new(false, config.clone()),
                help: Help::new(config.clone()),
                message: (false, {
                    if let Some(s) = config.global.find_action_str(TargetedAction::OpenHelp) {
                        format!("Welcome to Ampterm. Press {} at any point to see all of its keybindings.", s)
                    } else if let Some(s) =
                        config.global.find_action_str(TargetedAction::ToggleHelp)
                    {
                        format!("Welcome to Ampterm. Press {} at any point to see all of its keybindings.", s)
                    } else {
                        "Welcome to Ampterm.".to_string()
                    }
                }),
                config,
                state: CurrentlySelected::PlaylistList,
                current_mode: Mode::Normal,
                key_stack: vec![],
                popup: Popup::None,
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

        match &mut self.popup {
            Popup::None => {}
            Popup::Tasks => self.tasks.draw(frame, area),
            Popup::Help => self.help.draw(frame, area),
            Popup::PlaylistInfo(comp) => comp.draw(frame, area),
            Popup::MediaInfo(comp) => comp.draw(frame, area),
            Popup::SelectPlaylist(comp) => comp.draw(frame, area),
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
        let (is_err, msg) = self.message.clone();

        let mut msg_comp = Paragraph::new(msg).wrap(Wrap { trim: false });
        if is_err {
            msg_comp = msg_comp.fg(Color::Red);
        }
        frame.render_widget(msg_comp, text_areas[1]);
        frame.render_widget(
            Paragraph::new(self.key_stack.join(" ")).wrap(Wrap { trim: false }),
            text_areas[2],
        );
    }
}

impl HandlePlayer for MainScreen {
    fn handle_player(&mut self, pw: FromPlayerWorker) -> Option<Action> {
        let now_playing = self.now_playing.handle_player(pw.clone());
        let playqueue = self.playqueue.handle_player(pw);
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
        match dest {
            CompID::PlaylistList => self.pl_list.handle_query(dest, ticket, res),
            CompID::PlaylistQueue => self.pl_queue.handle_query(dest, ticket, res),
            CompID::NowPlaying | CompID::Lyrics | CompID::ImageComp => {
                self.now_playing.handle_query(dest, ticket, res)
            }
            CompID::PlayQueue => self.playqueue.handle_query(dest, ticket, res),
            CompID::MainScreen => {
                if let QueryStatus::Finished(body) = res {
                    if let ResponseType::GetPlaylists(pl) = body {
                        match pl {
                            Ok(p) => {
                                // User may close the popup before the request is finished
                                if let Popup::SelectPlaylist(popup) = &mut self.popup {
                                    popup.update_playlist(p);
                                }
                            }
                            Err(err) => {
                                self.message = (true, err);
                                self.popup = Popup::None;
                            }
                        };
                        None
                    } else {
                        unreachable!()
                    }
                } else {
                    None
                }
            }
            _ => unreachable!(),
        }
    }
}

impl HandleAction for MainScreen {
    fn handle_action(&mut self, action: TargetedAction) -> Option<Action> {
        self.key_stack.drain(..);
        match action {
            TargetedAction::PrepareAddToPlaylist(list) => {
                let len = list.len();
                let (popup, action) = SelectPlaylistPopup::new(
                    list,
                    self.config.local.select_playlist_popup.clone(),
                    format!("Add {} items to a playlist", len),
                );
                self.popup = Popup::SelectPlaylist(popup);
                Some(action)
            }
            TargetedAction::ViewPlaylistInfo(playlist) => {
                self.popup = Popup::PlaylistInfo(PlaylistInfo::new(
                    playlist,
                    self.config.local.popup.clone(),
                ));
                None
            }
            TargetedAction::ViewMediaInfo(media) => {
                self.popup =
                    Popup::MediaInfo(MediaInfo::new(media, self.config.local.popup.clone()));
                None
            }
            TargetedAction::ToggleHelp => {
                if matches!(self.popup, Popup::Help) {
                    self.popup = Popup::None;
                } else {
                    self.show_help();
                };
                None
            }
            TargetedAction::ClosePopup => {
                self.popup = Popup::None;
                None
            }
            TargetedAction::OpenHelp => {
                self.show_help();
                None
            }
            TargetedAction::Queue(_)
            | TargetedAction::Skip
            | TargetedAction::Previous
            | TargetedAction::Shuffle => self.playqueue.handle_action(action),
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
            TargetedAction::Info(msg) => {
                self.message = (false, msg);
                None
            }
            TargetedAction::Err(msg) => {
                self.message = (true, msg);
                None
            }
            _ => None,
        }
    }
}

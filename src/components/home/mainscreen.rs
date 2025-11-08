mod bpmtoy;
mod nowplaying;
mod playlistlist;
mod playlistqueue;
mod playqueue;
mod tasks;

use crate::{
    action::{
        useraction::{Global, Normal, UserAction},
        Action, FromPlayerWorker,
    },
    app::Mode,
    compid::CompID,
    components::{
        home::mainscreen::{bpmtoy::BPMToy, tasks::Tasks},
        traits::{
            component::Component, focusable::Focusable, ontick::OnTick,
            simplecomponent::SimpleComponent,
        },
    },
    config::Config,
    queryworker::{highlevelquery::HighLevelQuery, query::ToQueryWorker},
};
use color_eyre::Result;
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

#[derive(PartialEq)]
enum CurrentlySelected {
    PlaylistList,
    Playlist,
    PlayQueue,
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

impl MainScreen {
    fn propagate_to_focused_component(&mut self, action: Action) -> Result<Option<Action>> {
        if self.show_tasks {
            self.tasks.update(action)
        } else {
            match self.state {
                CurrentlySelected::PlaylistList => self.pl_list.update(action),
                CurrentlySelected::Playlist => self.pl_queue.update(action),
                CurrentlySelected::PlayQueue => self.playqueue.update(action),
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
                playqueue: PlayQueue::new(false),
                now_playing: NowPlaying::new(),
                tasks: Tasks::new(config.config.show_internal_tasks),
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
    }
}

impl Component for MainScreen {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        self.key_stack.push(format!(
            "{}{}",
            key.code.to_string(),
            if key.modifiers == KeyModifiers::NONE {
                ""
            } else {
                "+"
            },
        ));
        Ok(None)
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
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
        self.pl_list.draw(frame, listareas[0])?;
        self.pl_queue.draw(frame, listareas[1])?;
        self.playqueue.draw(frame, listareas[2])?;
        self.now_playing.draw(frame, bottom_areas[0])?;
        self.bpmtoy.draw(frame, bottom_areas[1])?;

        if self.show_tasks {
            self.tasks.draw(frame, area)?;
        }

        frame.render_widget(
            Paragraph::new(format!("[{}]", self.current_mode.to_string()))
                .wrap(Wrap { trim: false }),
            text_areas[0],
        );

        frame.render_widget(
            Paragraph::new(format!("Ampache {}", env!("CARGO_PKG_VERSION")))
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
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if matches!(action, Action::User(_)) {
            self.key_stack.drain(..);
        };
        match &action {
            Action::FromPlayerWorker(pw) => match pw {
                FromPlayerWorker::StateChange(_) => {
                    self.now_playing.update(action.clone());
                    self.playqueue.update(action)
                }
                FromPlayerWorker::Error(msg) | FromPlayerWorker::Message(msg) => {
                    self.message = msg.clone();
                    Ok(None)
                }
            },
            Action::ChangeMode(m) => {
                self.current_mode = *m;
                Ok(None)
            }
            Action::User(u) => match u {
                UserAction::Normal(normal) => match normal {
                    Normal::WindowLeft => {
                        self.state = match self.state {
                            CurrentlySelected::PlaylistList => CurrentlySelected::PlayQueue,
                            CurrentlySelected::PlayQueue => CurrentlySelected::Playlist,
                            CurrentlySelected::Playlist => CurrentlySelected::PlaylistList,
                        };
                        self.update_focus();
                        Ok(None)
                    }
                    Normal::WindowRight => {
                        self.state = match self.state {
                            CurrentlySelected::PlaylistList => CurrentlySelected::Playlist,
                            CurrentlySelected::Playlist => CurrentlySelected::PlayQueue,
                            CurrentlySelected::PlayQueue => CurrentlySelected::PlaylistList,
                        };
                        self.update_focus();
                        Ok(None)
                    }
                    _ => self.propagate_to_focused_component(action),
                },
                UserAction::Global(global) => match global {
                    Global::TapToBPM => self.bpmtoy.update(action),
                    Global::FocusPlaylistList => {
                        self.state = CurrentlySelected::PlaylistList;
                        self.update_focus();
                        Ok(None)
                    }
                    Global::FocusPlaylistQueue => {
                        self.state = CurrentlySelected::Playlist;
                        self.update_focus();
                        Ok(None)
                    }
                    Global::FocusPlayQueue => {
                        self.state = CurrentlySelected::PlayQueue;
                        self.update_focus();
                        Ok(None)
                    }
                    Global::EndKeySeq => Ok(None),
                    Global::OpenTasks => {
                        self.show_tasks = true;
                        Ok(None)
                    }
                    Global::CloseTasks => {
                        self.show_tasks = false;
                        Ok(None)
                    }
                    Global::ToggleTasks => {
                        self.show_tasks = !self.show_tasks;
                        Ok(None)
                    }
                },
                _ => self.propagate_to_focused_component(action),
            },
            Action::ToQueryWorker(req) => {
                let _ = self.tasks.register_task(req.clone());
                let mut results = vec![];

                for dest in &req.dest {
                    let res = match dest {
                        CompID::PlaylistList => self.pl_list.update(action.clone()),
                        CompID::PlaylistQueue => self.pl_queue.update(action.clone()),
                        CompID::PlayQueue => self.playqueue.update(action.clone()),
                        CompID::None => Ok(None),
                        _ => unreachable!("Action propagated to nonexistent component: {:?}", dest),
                    }?;
                    if let Some(a) = res {
                        results.push(a);
                    }
                }
                Ok(Some(Action::Multiple(results)))
            }
            Action::FromQueryWorker(res) => {
                let _ = self.tasks.unregister_task(res);
                let mut results = vec![];

                for dest in &res.dest {
                    let res = match dest {
                        CompID::PlaylistList => self.pl_list.update(action.clone()),
                        CompID::PlaylistQueue => self.pl_queue.update(action.clone()),
                        CompID::PlayQueue => self.playqueue.update(action.clone()),
                        CompID::None => Ok(None),
                        _ => unreachable!("Action propagated to nonexistent component: {:?}", dest),
                    }?;
                    if let Some(a) = res {
                        results.push(a);
                    }
                }
                Ok(Some(Action::Multiple(results)))
            }
            _ => {
                self.now_playing.update(action.clone());
                let results: Vec<Action> = [
                    self.pl_list.update(action.clone())?,
                    self.pl_queue.update(action.clone())?,
                    self.playqueue.update(action.clone())?,
                    self.tasks.update(action.clone())?,
                    self.bpmtoy.update(action)?,
                ]
                .into_iter()
                .filter_map(|a| a)
                .collect();

                Ok(Some(Action::Multiple(results)))
            }
        }
    }
}

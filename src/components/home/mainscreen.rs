mod nowplaying;
mod playlistlist;
mod playlistqueue;
mod queuelist;

use crate::{
    action::{
        useraction::{Normal, UserAction},
        Action, FromPlayerWorker,
    },
    app::Mode,
    components::{
        home::compid,
        traits::{component::Component, focusable::Focusable},
    },
    queryworker::{highlevelquery::HighLevelQuery, query::ToQueryWorker},
};
use color_eyre::Result;
use crossterm::event::{KeyEvent, KeyModifiers};
use nowplaying::NowPlaying;
use playlistlist::PlaylistList;
use playlistqueue::PlaylistQueue;
use queuelist::QueueList;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    widgets::{Paragraph, Wrap},
    Frame,
};

#[derive(PartialEq)]
enum CurrentlySelected {
    Playlists,
    PlaylistQueue,
    Queue,
}

pub struct MainScreen {
    state: CurrentlySelected,
    pl_list: PlaylistList,
    pl_queue: PlaylistQueue,
    now_playing: NowPlaying,
    queuelist: QueueList,
    message: String,
    key_stack: Vec<String>,
    current_mode: Mode,
}

impl MainScreen {
    pub fn new() -> (Self, Action) {
        (
            Self {
                state: CurrentlySelected::Playlists,
                current_mode: Mode::Normal,
                pl_list: PlaylistList::new(true),
                pl_queue: PlaylistQueue::new(false),
                queuelist: QueueList::new(false),
                now_playing: NowPlaying::new(),
                message: "You are now logged in.".to_string(),
                key_stack: vec![],
            },
            Action::Multiple(vec![
                Some(Action::ToQueryWorker(ToQueryWorker::new(
                    HighLevelQuery::ListPlaylists,
                ))),
                Some(Action::ChangeMode(Mode::Normal)),
            ]),
        )
    }
    fn update_focus(&mut self) {
        self.pl_list
            .set_enabled(self.state == CurrentlySelected::Playlists);
        self.pl_queue
            .set_enabled(self.state == CurrentlySelected::PlaylistQueue);
        self.queuelist
            .set_enabled(self.state == CurrentlySelected::Queue);
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
        let listareas = horizontal.split(areas[0]);
        let text_areas = text_layout.split(areas[2]);

        self.pl_list.draw(frame, listareas[0])?;
        self.pl_queue.draw(frame, listareas[1])?;
        self.queuelist.draw(frame, listareas[2])?;
        self.now_playing.draw(frame, areas[1])?;
        frame.render_widget(
            Paragraph::new(format!("[{}]", self.current_mode.to_string()))
                .wrap(Wrap { trim: false }),
            text_areas[0],
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
        if matches!(action, Action::EndKeySeq) {
            self.key_stack.drain(..);
        }
        match &action {
            Action::FromPlayerWorker(pw) => {
                if let FromPlayerWorker::PlayerError(msg) | FromPlayerWorker::PlayerMessage(msg) =
                    pw
                {
                    self.message = msg.clone();
                };
            }
            Action::User(UserAction::Normal(n)) => match n {
                Normal::WindowLeft => {
                    self.state = match self.state {
                        CurrentlySelected::Playlists => CurrentlySelected::Queue,
                        CurrentlySelected::Queue => CurrentlySelected::PlaylistQueue,
                        CurrentlySelected::PlaylistQueue => CurrentlySelected::Playlists,
                    };
                    self.update_focus();
                }
                Normal::WindowRight => {
                    self.state = match self.state {
                        CurrentlySelected::Playlists => CurrentlySelected::PlaylistQueue,
                        CurrentlySelected::PlaylistQueue => CurrentlySelected::Queue,
                        CurrentlySelected::Queue => CurrentlySelected::Playlists,
                    };
                    self.update_focus();
                }
                _ => {}
            },
            Action::ChangeMode(m) => {
                self.current_mode = *m;
            }
            _ => {}
        };
        match &action {
            Action::User(_) => match self.state {
                CurrentlySelected::Playlists => self.pl_list.update(action),
                CurrentlySelected::PlaylistQueue => self.pl_queue.update(action),
                CurrentlySelected::Queue => self.queuelist.update(action),
            },
            Action::FromQueryWorker(res) => match res.dest {
                compid::CompID::PlaylistList => self.pl_list.update(action.clone()),
                compid::CompID::PlaylistQueue => self.pl_queue.update(action.clone()),
                compid::CompID::QueueList => self.queuelist.update(action.clone()),
                _ => panic!("Invalid routing detected!"),
            },
            _ => Ok(Some(Action::Multiple(vec![
                self.pl_list.update(action.clone())?,
                self.pl_queue.update(action.clone())?,
                self.now_playing.update(action.clone())?,
                self.queuelist.update(action)?,
            ]))),
        }
    }
}

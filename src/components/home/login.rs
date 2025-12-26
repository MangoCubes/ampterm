use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use tui_textarea::{CursorMove, TextArea};

use crate::{
    action::action::{Action, TargetedAction},
    compid::CompID,
    components::{
        lib::checkbox::Checkbox,
        traits::{
            focusable::Focusable, handlequery::HandleQuery, handleraw::HandleRaw,
            renderable::Renderable,
        },
    },
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{setcredential::Credential, QueryStatus, ResponseType, ToQueryWorker},
    },
};

#[derive(Default, PartialEq)]
enum Status {
    #[default]
    Normal,
    /// Contains the ticket for SetCredential query
    Pending(usize),
    Error,
}
#[derive(Default, PartialEq)]
enum Mode {
    #[default]
    Url,
    Username,
    Password,
    LegacyToggle,
}

pub struct Login {
    username: TextArea<'static>,
    password: TextArea<'static>,
    url: TextArea<'static>,
    legacy: Checkbox,
    status_msg: Option<Vec<String>>,
    mode: Mode,
    status: Status,
}

impl Login {
    fn update_style(&mut self) {
        fn change_style(textarea: &mut TextArea<'_>, enable: bool, title: &'static str) {
            if enable {
                textarea.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
                textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
                textarea.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default())
                        .title(title),
                );
            } else {
                textarea.set_cursor_line_style(Style::default());
                textarea.set_cursor_style(Style::default());
                textarea.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::DarkGray))
                        .title(title),
                );
            }
        }
        change_style(
            &mut self.url,
            self.mode == Mode::Url && !matches!(self.status, Status::Pending(_)),
            "URL",
        );
        change_style(
            &mut self.username,
            self.mode == Mode::Username && !matches!(self.status, Status::Pending(_)),
            "Username",
        );
        change_style(
            &mut self.password,
            self.mode == Mode::Password && !matches!(self.status, Status::Pending(_)),
            "Password",
        );
        self.legacy.set_enabled(self.mode == Mode::LegacyToggle);
    }
    fn navigate(&mut self, up: bool) {
        if matches!(self.status, Status::Pending(_)) {
            return;
        }
        self.mode = if up {
            match self.mode {
                Mode::Url => Mode::LegacyToggle,
                Mode::Username => Mode::Url,
                Mode::Password => Mode::Username,
                Mode::LegacyToggle => Mode::Password,
            }
        } else {
            match self.mode {
                Mode::Url => Mode::Username,
                Mode::Username => Mode::Password,
                Mode::Password => Mode::LegacyToggle,
                Mode::LegacyToggle => Mode::Url,
            }
        };
        self.update_style();
    }
    // Submit current form to the server
    // This function never fails and handles errors from attempt_login
    fn submit(&mut self) -> Option<Action> {
        let url = self.url.lines()[0].clone();
        let username = self.username.lines()[0].clone();
        let password = self.password.lines()[0].clone();
        let q = ToQueryWorker::new(HighLevelQuery::SetCredential(Credential::Password {
            url,
            secure: true,
            username,
            password,
            legacy: self.legacy.get_toggle(),
        }));
        self.status = Status::Pending(q.ticket);
        self.status_msg = Some(vec!["Logging in...".to_string()]);
        self.update_style();
        Some(Action::ToQuery(q))
    }
    pub fn new(msg: Option<Vec<String>>) -> Self {
        let mut res = Self {
            username: TextArea::default(),
            password: TextArea::default(),
            url: TextArea::new(vec!["https://".to_string()]),
            mode: Mode::default(),
            status: Status::default(),
            status_msg: msg,
            legacy: Checkbox::new(false, false, "Use legacy auth instead".to_string()),
        };
        res.url.move_cursor(CursorMove::End);
        res.password.set_mask_char('*');
        res.update_style();
        res
    }
}

impl Renderable for Login {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let [horizontal] = Layout::horizontal([Constraint::Percentage(50)])
            .flex(Flex::Center)
            .areas(area);
        let [centered] = Layout::vertical([Constraint::Percentage(50)])
            .flex(Flex::Center)
            .areas(horizontal);
        let layout = Layout::default().constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let areas = layout.split(centered);
        frame.render_widget(&self.url, areas[0]);
        frame.render_widget(&self.username, areas[1]);
        frame.render_widget(&self.password, areas[2]);
        frame.render_widget(
            Paragraph::new(vec![
                Line::raw("Enter: Submit"),
                Line::raw("Space: Toggle checkbox"),
                Line::raw("Tab or arrow keys: Navigate"),
            ])
            .centered(),
            areas[4],
        );
        self.legacy.draw(frame, areas[3]);
        if let Some(msg) = &self.status_msg {
            let text: Vec<Line> = msg.iter().map(|l| Line::raw(l)).collect();
            frame.render_widget(
                Paragraph::new(text).centered().wrap(Wrap { trim: false }),
                areas[5],
            );
        }
    }
}

impl HandleQuery for Login {
    fn handle_query(&mut self, _dest: CompID, ticket: usize, res: QueryStatus) -> Option<Action> {
        if let Status::Pending(t) = self.status {
            if ticket == t {
                if let QueryStatus::Finished(ResponseType::Ping(Err(msg))) = res {
                    self.status_msg = Some(vec!["Failed to log in! Error:".to_string(), msg]);
                    self.status = Status::Error;
                    self.update_style();
                }
            }
        }
        None
    }
    // fn handle_query(&mut self, action: QueryAction) -> Option<Action> {
    //     if let QueryAction::FromQueryWorker(res) = action {
    //     }
    //     None
    // }
}

impl HandleRaw for Login {
    fn handle_raw(&mut self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Up | KeyCode::BackTab | KeyCode::Left => {
                self.navigate(true);
                None
            }
            KeyCode::Down | KeyCode::Tab | KeyCode::Right => {
                self.navigate(false);
                None
            }
            KeyCode::Esc => Some(Action::Targeted(TargetedAction::Quit)),
            KeyCode::Enter => {
                if !matches!(self.status, Status::Pending(_)) {
                    self.submit()
                } else {
                    None
                }
            }
            _ => match self.mode {
                Mode::Url => {
                    self.url.input(key);
                    None
                }
                Mode::Username => {
                    self.username.input(key);
                    None
                }
                Mode::Password => {
                    self.password.input(key);
                    None
                }
                Mode::LegacyToggle => self.legacy.handle_raw(key),
            },
        }
    }
}

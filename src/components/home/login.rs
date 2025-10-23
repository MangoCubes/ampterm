use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use tui_textarea::{CursorMove, TextArea};

use crate::{
    action::Action,
    components::{lib::checkbox::Checkbox, traits::focusable::Focusable},
    config::Config,
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{setcredential::Credential, ToQueryWorker},
    },
};

use super::Component;
#[derive(Default, PartialEq)]
enum Status {
    #[default]
    Normal,
    Pending,
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
    config: Config,
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
            self.mode == Mode::Url && self.status != Status::Pending,
            "URL",
        );
        change_style(
            &mut self.username,
            self.mode == Mode::Username && self.status != Status::Pending,
            "Username",
        );
        change_style(
            &mut self.password,
            self.mode == Mode::Password && self.status != Status::Pending,
            "Password",
        );
        self.legacy.set_enabled(self.mode == Mode::LegacyToggle);
    }
    fn navigate(&mut self, up: bool) -> Result<Option<Action>> {
        if self.status == Status::Pending {
            return Ok(None);
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
        Ok(None)
    }
    // Submit current form to the server
    // This function never fails and handles errors from attempt_login
    fn submit(&mut self) -> Result<Option<Action>> {
        let url = self.url.lines()[0].clone();
        let username = self.username.lines()[0].clone();
        let password = self.password.lines()[0].clone();
        self.status = Status::Pending;
        self.status_msg = Some(vec!["Logging in...".to_string()]);
        self.update_style();
        Ok(Some(Action::Multiple(vec![
            Action::ToQueryWorker(ToQueryWorker::new(HighLevelQuery::SetCredential(
                Credential::Password {
                    url,
                    secure: true,
                    username,
                    password,
                    legacy: self.legacy.get_toggle(),
                },
            ))),
            Action::ToQueryWorker(ToQueryWorker::new(HighLevelQuery::CheckCredentialValidity)),
        ])))
    }
    pub fn new(msg: Option<Vec<String>>, config: Config) -> (Self, Action) {
        let mut res = Self {
            username: TextArea::default(),
            password: TextArea::default(),
            url: TextArea::new(vec!["https://".to_string()]),
            mode: Mode::default(),
            status: Status::default(),
            status_msg: msg,
            config,
            legacy: Checkbox::new(false, false, "Use legacy auth instead".to_string()),
        };
        res.url.move_cursor(CursorMove::End);
        res.password.set_mask_char('*');
        res.update_style();
        (res, Action::ChangeMode(crate::app::Mode::Insert))
    }
}

impl Component for Login {
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Up | KeyCode::BackTab | KeyCode::Left => self.navigate(true),
            KeyCode::Down | KeyCode::Tab | KeyCode::Right => self.navigate(false),
            KeyCode::Esc => Ok(Some(Action::Quit)),
            KeyCode::Enter => {
                return self.submit();
            }
            _ => match self.mode {
                Mode::Url => {
                    self.url.input(key);
                    Ok(None)
                }
                Mode::Username => {
                    self.username.input(key);
                    Ok(None)
                }
                Mode::Password => {
                    self.password.input(key);
                    Ok(None)
                }
                Mode::LegacyToggle => self.legacy.handle_key_event(key),
            },
        }
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
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
                Line::raw("Tab or arrow keys: Navigate"),
            ])
            .centered(),
            areas[4],
        );
        self.legacy.draw(frame, areas[3])?;
        if let Some(msg) = &self.status_msg {
            let text: Vec<Line> = msg.iter().map(|l| Line::raw(l)).collect();
            frame.render_widget(
                Paragraph::new(text).centered().wrap(Wrap { trim: false }),
                areas[5],
            );
        }
        Ok(())
    }
}

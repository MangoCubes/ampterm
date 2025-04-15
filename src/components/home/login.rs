use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::{CursorMove, TextArea};

use crate::{
    action::Action,
    config::Config,
    queryworker::{
        query::{
            login::{Credentials, LoginQuery},
            Query,
        },
        response::{login::LoginResponse, Response},
    },
    trace_dbg,
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
}

pub struct Login {
    username: TextArea<'static>,
    password: TextArea<'static>,
    url: TextArea<'static>,
    status_msg: Option<Vec<String>>,
    action_tx: UnboundedSender<Action>,
    mode: Mode,
    status: Status,
    config: Config,
}

impl Login {
    fn set_error(&mut self, msg: String) {
        self.status_msg = Some(vec![msg]);
        self.update_style();
    }
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
    }
    fn navigate(&mut self, up: bool) -> Result<()> {
        if self.status == Status::Pending {
            return Ok(());
        }
        self.mode = if up {
            match self.mode {
                Mode::Url => Mode::Password,
                Mode::Username => Mode::Url,
                Mode::Password => Mode::Username,
            }
        } else {
            match self.mode {
                Mode::Url => Mode::Username,
                Mode::Username => Mode::Password,
                Mode::Password => Mode::Url,
            }
        };
        self.update_style();
        Ok(())
    }
    // Submit current form to the server
    // This function never fails and handles errors from attempt_login
    fn submit(&mut self) -> Result<()> {
        let url = self.url.lines()[0].clone();
        let username = self.username.lines()[0].clone();
        let password = self.password.lines()[0].clone();
        self.status = Status::Pending;
        self.status_msg = Some(vec!["Logging in...".to_string()]);
        self.update_style();
        let action = Action::Query(Query::Login(LoginQuery::Login(Credentials::new(
            url,
            username,
            password,
            self.config.config.use_legacy_auth,
        ))));
        self.action_tx.send(action)?;
        Ok(())
    }
    pub fn new(action_tx: UnboundedSender<Action>, config: Config) -> Self {
        let mut res = Self {
            username: TextArea::default(),
            password: TextArea::default(),
            url: TextArea::new(vec!["https://".to_string()]),
            action_tx,
            mode: Mode::default(),
            status: Status::default(),
            status_msg: None,
            config,
        };
        res.url.move_cursor(CursorMove::End);
        res.password.set_mask_char('*');
        res.update_style();
        res
    }
}

impl Component for Login {
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::Response(r) = action {
            if let Response::Login(l) = r {
                trace_dbg!("Received Login response!");
                match l {
                    LoginResponse::InvalidURL => self.set_error("Invalid URL. Please check if this endpoint is running OpenSubsonic-compatible music server.".to_string()),
                    LoginResponse::InvalidCredentials => self.set_error("Your login is invalid. Please check your username or password.".to_string()),
                    LoginResponse::Other(err) => self.set_error(format!("Connection failed: {}", err)),
                    LoginResponse::FailedPing => self.set_error("Failed to ping the server. Please double check your URL.".to_string()),
                    LoginResponse::Success => {
                        self.status = Status::Normal;
                        self.update_style();
                        return Ok(None);
                    },
                };
                self.status = Status::Error;
                self.update_style();
            };
        };
        Ok(None)
    }
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Up => self.navigate(true),
            KeyCode::Down => self.navigate(false),
            KeyCode::Enter => self.submit(),
            KeyCode::Tab => self.navigate(false),
            KeyCode::BackTab => self.navigate(true),
            _ => match self.mode {
                Mode::Url => {
                    self.url.input(key);
                    Ok(())
                }
                Mode::Username => {
                    self.username.input(key);
                    Ok(())
                }
                Mode::Password => {
                    self.password.input(key);
                    Ok(())
                }
            },
        }?;
        Ok(None)
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
            areas[3],
        );
        if let Some(msg) = &self.status_msg {
            let text: Vec<Line> = msg.iter().map(|l| Line::raw(l)).collect();
            frame.render_widget(
                Paragraph::new(text).centered().wrap(Wrap { trim: false }),
                areas[4],
            );
        }
        Ok(())
    }
}

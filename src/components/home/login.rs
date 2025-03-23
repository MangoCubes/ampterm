use crate::action::loginaction::LoginAction;
use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use sunk::Client;
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::{CursorMove, TextArea};

use super::Component;
use crate::action::Action;
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
}

fn attempt_login(site: String, username: String, password: String) -> Result<(), sunk::Error> {
    let client = Client::new(site.as_str(), username.as_str(), password.as_str())?;
    client.ping()?;
    Ok(())
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
        let site = self.url.lines()[0].clone();
        let username = self.username.lines()[0].clone();
        let password = self.password.lines()[0].clone();
        self.status = Status::Pending;
        self.status_msg = Some(vec!["Logging in...".to_string()]);
        self.update_style();
        match attempt_login(site.clone(), username.clone(), password.clone()) {
            Ok(()) => {
                self.action_tx.send(Action::Login(LoginAction::Success(
                    site, username, password,
                )))?;
            }
            Err(e) => println!("{}", e),
        };
        Ok(())
    }
    pub fn new(action_tx: UnboundedSender<Action>) -> Self {
        let mut res = Self {
            username: TextArea::default(),
            password: TextArea::default(),
            url: TextArea::new(vec!["https://".to_string()]),
            action_tx,
            mode: Mode::default(),
            status: Status::default(),
            status_msg: None,
        };
        res.url.move_cursor(CursorMove::End);
        res.password.set_mask_char('*');
        res.update_style();
        res
    }
}

impl Component for Login {
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<Action>> {
        let _ = match key.code {
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
        };
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
            frame.render_widget(Paragraph::new(text).centered(), areas[4]);
        }
        Ok(())
    }
}

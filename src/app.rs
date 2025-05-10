use color_eyre::Result;

use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self};
use tracing::{debug, info};

use crate::{
    action::Action,
    components::{home::Home, Component},
    config::Config,
    playerworker::{player::PlayerAction, PlayerWorker},
    queryworker::{query::Query, QueryWorker},
    tui::{Event, Tui},
};

pub struct App {
    config: Config,
    tick_rate: f64,
    frame_rate: f64,
    component: Box<dyn Component>,
    should_quit: bool,
    should_suspend: bool,
    mode: Mode,
    key_stack: Vec<KeyEvent>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    query_tx: mpsc::UnboundedSender<Query>,
    player_tx: mpsc::UnboundedSender<PlayerAction>,
    stream: rodio::OutputStream,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Common,
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let config = Config::new()?;
        let mut qw = QueryWorker::new(action_tx.clone(), config.clone());
        let query_tx = qw.get_tx();
        // Start query worker
        tokio::spawn(async move { qw.run().await });

        let (stream, handle) = rodio::OutputStream::try_default().unwrap();
        let mut pw = PlayerWorker::new(handle, action_tx.clone(), config.clone());
        let player_tx = pw.get_tx();
        // Start query worker
        tokio::spawn(async move { pw.run().await });
        Ok(Self {
            tick_rate,
            frame_rate,
            component: Box::new(Home::new(action_tx.clone(), config.clone())),
            should_quit: false,
            should_suspend: false,
            config,
            mode: Mode::Common,
            key_stack: Vec::new(),
            action_tx,
            action_rx,
            query_tx,
            player_tx,
            stream,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            // .mouse(true) // uncomment this line to enable mouse support
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        tui.enter()?;

        self.component.init(tui.size()?)?;
        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        // let _ = tokio::join!(self.query_thread);
        tui.exit()?;
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx.send(Action::Quit)?,
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,
            _ => {}
        }
        if let Some(action) = self.component.handle_events(event.clone())? {
            action_tx.send(action)?;
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let action_tx = self.action_tx.clone();
        let Some(keymap) = self.config.keybindings.get(&self.mode) else {
            return Ok(());
        };

        self.key_stack.push(key);

        if let Some(action) = keymap.get(&self.key_stack) {
            info!("Got action: {action:?}");
            action_tx.send(action.clone())?;
            self.key_stack.drain(..);
        } else if let Some(action) = keymap.get(&vec![key]) {
            info!("Got action: {action:?}");
            action_tx.send(action.clone())?;
            self.key_stack.drain(..);
        }
        Ok(())
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
            };

            match &action {
                Action::EndKeySeq => {
                    self.key_stack.drain(..);
                }
                Action::Play => {
                    self.player_tx.send(PlayerAction::Continue)?;
                    if let Some(ret) = self.component.update(action)? {
                        self.action_tx.send(ret)?
                    }
                }
                Action::Pause => {
                    self.player_tx.send(PlayerAction::Pause)?;
                    if let Some(ret) = self.component.update(action)? {
                        self.action_tx.send(ret)?
                    }
                }
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Resize(w, h) => self.handle_resize(tui, *w, *h)?,
                Action::Render => self.render(tui)?,
                Action::Query(q) => {
                    self.query_tx.send(q.clone())?;
                    if let Some(ret) = self.component.update(action)? {
                        self.action_tx.send(ret)?
                    }
                }
                Action::Player(a) => {
                    self.player_tx.send(a.clone())?;
                    if let Some(ret) = self.component.update(action)? {
                        self.action_tx.send(ret)?
                    }
                }
                _ => {
                    if let Some(ret) = self.component.update(action)? {
                        self.action_tx.send(ret)?
                    }
                }
            };
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            if let Err(err) = self.component.draw(frame, frame.area()) {
                let _ = self
                    .action_tx
                    .send(Action::Error(format!("Failed to draw: {:?}", err)));
            }
        })?;
        Ok(())
    }
}

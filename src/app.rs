use std::sync::atomic::{AtomicUsize, Ordering};

use color_eyre::Result;

use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use rodio::{
    cpal::{self, traits::HostTrait, SupportedBufferSize},
    DeviceTrait, SupportedStreamConfig,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self};
use tracing::{debug, info};

use crate::{
    action::Action,
    components::{home::Home, traits::component::Component},
    config::Config,
    playerworker::{player::ToPlayerWorker, PlayerWorker},
    queryworker::{query::ToQueryWorker, QueryWorker},
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
    query_tx: mpsc::UnboundedSender<ToQueryWorker>,
    player_tx: mpsc::UnboundedSender<ToPlayerWorker>,
    stream: rodio::OutputStream,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    Common,
    #[default]
    Normal,
    Visual,
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

impl App {
    pub fn get_id() -> usize {
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }
    pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let config = Config::new()?;
        let mut qw = QueryWorker::new(action_tx.clone(), config.clone());
        let query_tx = qw.get_tx();
        // Start query worker
        tokio::spawn(async move { qw.run().await });

        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let default_config = device.default_output_config()?;
        let (stream, handle) = rodio::OutputStream::try_from_device_config(
            &device,
            SupportedStreamConfig::new(
                default_config.channels(),
                default_config.sample_rate(),
                SupportedBufferSize::Range {
                    min: 4096,
                    max: 4096,
                },
                default_config.sample_format(),
            ),
        )
        .unwrap();
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
            mode: Mode::default(),
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
                Action::ToPlayerWorker(pw) => {
                    self.player_tx.send(pw.clone())?;
                }
                Action::ToQueryWorker(qw) => {
                    self.query_tx.send(qw.clone())?;
                }
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Resize(w, h) => self.handle_resize(tui, *w, *h)?,
                Action::Render => self.render(tui)?,
                Action::ChangeMode(mode) => {
                    self.mode = *mode;
                }
                _ => {}
            };
            if let Some(ret) = self.component.update(action)? {
                debug!("Got {ret:?} as a response");
                self.action_tx.send(ret)?
            }
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

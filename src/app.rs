use color_eyre::Result;

use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use tokio::sync::mpsc::{self};
use tracing::debug;

use crate::{
    action::action::{Action, Mode},
    components::{
        home::Home,
        traits::{
            handleaction::HandleAction,
            handlekeyseq::{HandleKeySeq, KeySeqResult},
            ontick::OnTick,
            renderable::Renderable,
        },
    },
    config::Config,
    playerworker::player::ToPlayerWorker,
    queryworker::query::ToQueryWorker,
    tui::{Event, Tui},
};

pub struct App {
    config: Config,
    tick_rate: f64,
    frame_rate: f64,
    component: Home,
    should_quit: bool,
    should_suspend: bool,
    mode: Mode,
    key_stack: Vec<KeyEvent>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    query_tx: mpsc::UnboundedSender<ToQueryWorker>,
    player_tx: mpsc::UnboundedSender<ToPlayerWorker>,
}

impl App {
    pub fn new(
        config: Config,
        action_tx: mpsc::UnboundedSender<Action>,
        action_rx: mpsc::UnboundedReceiver<Action>,
        query_tx: mpsc::UnboundedSender<ToQueryWorker>,
        player_tx: mpsc::UnboundedSender<ToPlayerWorker>,
        tick_rate: f64,
        frame_rate: f64,
    ) -> Result<Self> {
        let (component, actions) = Home::new(config.clone());
        actions.into_iter().for_each(|a| {
            let _ = action_tx.send(a);
        });
        Ok(Self {
            tick_rate,
            frame_rate,
            component,
            should_quit: false,
            should_suspend: false,
            config,
            mode: Mode::Normal,
            key_stack: Vec::new(),
            action_tx,
            action_rx,
            query_tx,
            player_tx,
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
            self.handle_actions(&mut tui).await?;
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
            Event::Tick => self.component.on_tick(),
            Event::Render => self.render(tui)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,
            _ => {}
        }
        Ok(())
    }

    fn find_global_action(&self, key: KeyEvent) -> Option<KeySeqResult> {
        match self.config.global.get(&self.key_stack) {
            // Test global map
            Some(a) => Some(KeySeqResult::ActionNeeded(Action::Targeted(a.clone()))),
            None => match self.config.global.get(&vec![key]) {
                // Test global map single key
                Some(a) => Some(KeySeqResult::ActionNeeded(Action::Targeted(a.clone()))),
                None => None,
            },
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        self.key_stack.push(key);

        let res = if let Some(r) = self.component.handle_key_seq(&self.key_stack) {
            r
        } else if let Some(r) = self.find_global_action(key) {
            r
        } else {
            return Ok(());
        };

        self.key_stack.drain(..);

        if let KeySeqResult::ActionNeeded(action) = res {
            self.action_tx.send(action.clone())?;
        }
        Ok(())
    }

    async fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            debug!("{action:?}");
            match action {
                Action::Multiple(actions) => {
                    for a in actions {
                        self.action_tx.send(a)?
                    }
                }
                Action::Targeted(targeted_action) => {
                    if let Some(more) = self.component.handle_action(targeted_action) {
                        debug!("Got {more:?} as a response");
                        self.action_tx.send(more)?
                    }
                }
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Quit => self.should_quit = true,
                Action::ToPlayerWorker(action) => self.player_tx.send(action)?,
                Action::ToQueryWorker(action) => self.query_tx.send(action)?,
                Action::ChangeMode(mode) => self.mode = mode,
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
        tui.draw(|frame| self.component.draw(frame, frame.area()))?;
        Ok(())
    }
}

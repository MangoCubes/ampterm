use color_eyre::Result;

use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use tokio::sync::mpsc::{self};
use tracing::debug;

use crate::{
    action::action::{Action, Mode, QueryAction, TargetedAction},
    components::{
        home::Home,
        traits::{
            handleaction::HandleAction,
            handlekeyseq::{KeySeqResult, PassKeySeq},
            handlemode::HandleMode,
            handlequery::HandleQuery,
            handleraw::HandleRaw,
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
    key_stack: Vec<KeyEvent>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    query_tx: mpsc::UnboundedSender<ToQueryWorker>,
    player_tx: mpsc::UnboundedSender<ToPlayerWorker>,
    mode: Mode,
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
            key_stack: Vec::new(),
            action_tx,
            action_rx,
            query_tx,
            player_tx,
            mode: Mode::Normal,
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
                action_tx.send(Action::Targeted(TargetedAction::Resume))?;
                action_tx.send(Action::Targeted(TargetedAction::ClearScreen))?;
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
            Event::Quit => action_tx.send(Action::Targeted(TargetedAction::Quit))?,
            Event::Tick => self.component.on_tick(),
            Event::Render => self.render(tui)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if self.mode == Mode::Insert {
            if let Some(action) = self.component.handle_raw(key) {
                self.action_tx.send(action)?;
            }
        } else {
            self.key_stack.push(key);

            let res = if let Some(r) = self.component.handle_key_seq(&self.key_stack) {
                r
            } else if let Some(r) = self.config.global.get(&self.key_stack) {
                KeySeqResult::ActionNeeded(Action::Targeted(r.clone()))
            } else {
                let single = &vec![key];
                if let Some(r) = self.component.handle_key_seq(single) {
                    r
                } else if let Some(r) = self.config.global.get(single) {
                    KeySeqResult::ActionNeeded(Action::Targeted(r.clone()))
                } else {
                    return Ok(());
                }
            };

            self.key_stack.drain(..);

            if let KeySeqResult::ActionNeeded(action) = res {
                self.action_tx.send(action)?;
            }
        };
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
                Action::Targeted(targeted_action) => match targeted_action {
                    TargetedAction::Suspend => self.should_suspend = true,
                    TargetedAction::Resume => self.should_suspend = false,
                    TargetedAction::ClearScreen => tui.terminal.clear()?,
                    TargetedAction::Quit => self.should_quit = true,
                    TargetedAction::Play => {
                        let _ = self.player_tx.send(ToPlayerWorker::Resume);
                    }
                    TargetedAction::Pause => {
                        let _ = self.player_tx.send(ToPlayerWorker::Pause);
                    }
                    TargetedAction::PlayOrPause => {
                        let _ = self.player_tx.send(ToPlayerWorker::ResumeOrPause);
                    }
                    TargetedAction::ChangeVolume(delta) => {
                        let _ = self.player_tx.send(ToPlayerWorker::ChangeVolume(delta));
                    }
                    TargetedAction::GoToStart => {
                        let _ = self.player_tx.send(ToPlayerWorker::GoToStart);
                    }
                    TargetedAction::EndKeySeq => {
                        self.key_stack.drain(..);
                        if let Some(more) = self.component.handle_action(targeted_action) {
                            debug!("Got {more:?} as a response");
                            self.action_tx.send(more)?
                        }
                    }
                    _ => {
                        if let Some(more) = self.component.handle_action(targeted_action) {
                            debug!("Got {more:?} as a response");
                            self.action_tx.send(more)?
                        }
                    }
                },
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::ChangeMode(mode) => {
                    self.mode = mode;
                    self.component.handle_mode(mode);
                }
                Action::Query(query_action) => {
                    match &query_action {
                        QueryAction::ToPlayerWorker(to_player_worker) => {
                            self.player_tx.send(to_player_worker.clone())?
                        }
                        QueryAction::ToQueryWorker(to_query_worker) => {
                            self.query_tx.send(to_query_worker.clone())?
                        }
                        _ => {}
                    }
                    debug!("Handling query: {query_action:?}");
                    if let Some(more) = self.component.handle_query(query_action) {
                        debug!("Got {more:?} as a response");
                        self.action_tx.send(more)?
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
        tui.draw(|frame| self.component.draw(frame, frame.area()))?;
        Ok(())
    }
}

mod delayer;
use color_eyre::Result;

use crossterm::event::KeyEvent;

use ratatui::prelude::Rect;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::debug;

use crate::{
    action::action::{Action, Mode, TargetedAction},
    app::delayer::Delayer,
    components::{
        home::Home,
        traits::{
            handleaction::HandleAction,
            handlekeyseq::{KeySeqResult, PassKeySeq},
            handlemode::HandleMode,
            handleplayer::HandlePlayer,
            handlequery::HandleQuery,
            handleraw::HandleRaw,
            ontick::OnTick,
            renderable::Renderable,
        },
    },
    config::Config,
    playerworker::player::{FromPlayerWorker, ToPlayerWorker},
    queryworker::query::{QueryStatus, ToQueryWorker},
    tui::{Event, Tui},
};

pub struct App {
    config: Config,
    component: Home,
    should_quit: bool,
    should_suspend: bool,
    key_stack: Vec<KeyEvent>,
    action_tx: UnboundedSender<Action>,
    action_rx: UnboundedReceiver<Action>,
    query_tx: UnboundedSender<ToQueryWorker>,
    player_tx: UnboundedSender<ToPlayerWorker>,
    mode: Mode,
    mpris_tx: UnboundedSender<FromPlayerWorker>,
    tui: Tui,
    delayer: Delayer,
    #[cfg(test)]
    debug_tx: UnboundedSender<bool>,
}

impl App {
    pub fn new(
        config: Config,
        action_tx: UnboundedSender<Action>,
        action_rx: UnboundedReceiver<Action>,
        mpris_tx: UnboundedSender<FromPlayerWorker>,
        query_tx: UnboundedSender<ToQueryWorker>,
        player_tx: UnboundedSender<ToPlayerWorker>,
        tick_rate: f64,
        frame_rate: f64,
        #[cfg(test)] debug_tx: UnboundedSender<bool>,
    ) -> Result<Self> {
        let (component, action) = Home::new(config.clone());
        let _ = action_tx.send(action);
        Ok(Self {
            tui: Tui::new()?.tick_rate(tick_rate).frame_rate(frame_rate),
            mpris_tx,
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
            delayer: Delayer::new(),
            #[cfg(test)]
            debug_tx,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.tui.enter()?;
        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events().await?;
            self.handle_actions().await?;
            if self.should_suspend {
                self.tui.suspend()?;
                action_tx.send(Action::Targeted(TargetedAction::Resume))?;
                action_tx.send(Action::Targeted(TargetedAction::ClearScreen))?;
                // tui.mouse(true);
                self.tui.enter()?;
            } else if self.should_quit {
                self.tui.stop()?;
                break;
            }
        }
        // let _ = tokio::join!(self.query_thread);
        self.tui.exit()?;
        Ok(())
    }

    async fn handle_events(&mut self) -> Result<()> {
        let Some(event) = self.tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx.send(Action::Targeted(TargetedAction::Quit))?,
            Event::Tick => {
                self.component.on_tick();
                while let Some(q) = self.delayer.on_tick() {
                    self.query_tx.send(q)?;
                }
            }
            Event::Render => self.render()?,
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

    async fn handle_actions(&mut self) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            match action {
                #[cfg(test)]
                Action::TestKey(event) => {
                    self.handle_key_event(event).unwrap();
                    self.render().unwrap();
                }
                #[cfg(test)]
                Action::TestKeys(events) => {
                    events
                        .into_iter()
                        .for_each(|k| self.handle_key_event(k).unwrap());
                    // Ensure the UI is rendered properly just before transmitting response
                    self.render().unwrap();
                }
                #[cfg(test)]
                Action::Snapshot(name) => {
                    self.render().unwrap();
                    insta::assert_snapshot!(name, self.tui.backend());
                    self.debug_tx.send(true).unwrap();
                }
                Action::Multiple(actions) => {
                    for a in actions {
                        self.action_tx.send(a)?
                    }
                }
                Action::Targeted(targeted_action) => match targeted_action {
                    TargetedAction::Suspend => self.should_suspend = true,
                    TargetedAction::Resume => self.should_suspend = false,
                    TargetedAction::ClearScreen => self.tui.terminal.clear()?,
                    TargetedAction::Quit => self.should_quit = true,
                    TargetedAction::Play => self.player_tx.send(ToPlayerWorker::Resume)?,
                    TargetedAction::Pause => self.player_tx.send(ToPlayerWorker::Pause)?,
                    TargetedAction::Stop => self.player_tx.send(ToPlayerWorker::Stop)?,
                    TargetedAction::PlayOrPause => {
                        self.player_tx.send(ToPlayerWorker::ResumeOrPause)?
                    }
                    TargetedAction::ChangeVolume(delta) => {
                        self.player_tx.send(ToPlayerWorker::ChangeVolume(delta))?
                    }

                    TargetedAction::SetVolume(to) => {
                        self.player_tx.send(ToPlayerWorker::SetVolume(to))?
                    }
                    TargetedAction::ChangeSpeed(delta) => {
                        self.player_tx.send(ToPlayerWorker::ChangeSpeed(delta))?
                    }
                    TargetedAction::SetSpeed(to) => {
                        self.player_tx.send(ToPlayerWorker::SetSpeed(to))?
                    }
                    TargetedAction::ChangePosition(by) => {
                        if let Some(more) = self.component.handle_action(targeted_action) {
                            self.action_tx.send(more)?
                        }
                        self.player_tx.send(ToPlayerWorker::ChangePosition(by))?
                    }
                    TargetedAction::SetPosition(to) => {
                        if let Some(more) = self.component.handle_action(targeted_action) {
                            self.action_tx.send(more)?
                        }
                        self.player_tx.send(ToPlayerWorker::SetPosition(to))?
                    }
                    TargetedAction::GoToStart => self.player_tx.send(ToPlayerWorker::GoToStart)?,
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
                Action::Resize(w, h) => self.handle_resize(w, h)?,
                Action::ChangeMode(mode) => {
                    self.mode = mode;
                    self.component.handle_mode(mode);
                }

                Action::ToQueryDelayed((query, delay)) => {
                    for d in &query.dest {
                        if let Some(more) = self.component.handle_query(
                            d.clone(),
                            query.ticket,
                            QueryStatus::Requested(query.query.clone()),
                        ) {
                            self.action_tx.send(more)?
                        }
                    }
                    self.delayer.queue_query(query, delay);
                }
                Action::ToQuery(query) => {
                    for d in &query.dest {
                        if let Some(more) = self.component.handle_query(
                            d.clone(),
                            query.ticket,
                            QueryStatus::Requested(query.query.clone()),
                        ) {
                            self.action_tx.send(more)?
                        }
                    }
                    self.query_tx.send(query)?
                }
                Action::ToPlayer(to_player_worker) => self.player_tx.send(to_player_worker)?,
                Action::FromPlayer(pw) => {
                    let _ = self.mpris_tx.send(pw.clone());
                    if let Some(more) = self.component.handle_player(pw) {
                        self.action_tx.send(more)?
                    }
                }
                Action::FromQuery { dest, ticket, res } => {
                    let more_actions: Vec<Action> = dest
                        .into_iter()
                        .filter_map(|c| self.component.handle_query(c, ticket, res.clone()))
                        .collect();
                    for a in more_actions {
                        self.action_tx.send(a)?
                    }
                }
            };
        }
        Ok(())
    }

    fn handle_resize(&mut self, w: u16, h: u16) -> Result<()> {
        self.tui.resize(Rect::new(0, 0, w, h))?;
        self.render()?;
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        self.tui
            .draw(|frame| self.component.draw(frame, frame.area()))?;
        Ok(())
    }
}

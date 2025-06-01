pub mod player;
pub mod streamerror;
pub mod streamreader;

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use color_eyre::Result;
use player::{PlayerAction, QueueLocation};
use rodio::{OutputStreamHandle, Sink};
use streamerror::StreamError;
use streamreader::StreamReader;
use tokio::select;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::action::getplaylist::Media;
use crate::action::Action;
use crate::config::Config;
use crate::queryworker::query::Query;
use crate::trace_dbg;

enum WorkerState {
    // The fetched file is played
    Playing {
        token: CancellationToken,
        item: Media,
    },
    // The file URL is being fetched
    Loading {
        item: Media,
    },
    // Nothing is in the queue, and there are no items being played right now
    Idle,
}

pub struct PlayerWorker {
    state: WorkerState,
    player_tx: UnboundedSender<PlayerAction>,
    player_rx: UnboundedReceiver<PlayerAction>,
    action_tx: UnboundedSender<Action>,
    should_quit: bool,
    config: Config,
    sink: Arc<Sink>,
    queue: VecDeque<Media>,
}

impl PlayerWorker {
    fn send_playlist_state(&mut self) {
        match &self.state {
            WorkerState::Playing { token: _, item } | WorkerState::Loading { item } => {
                let q = self.queue.clone().into();
                let _ = self.action_tx.send(Action::InQueue {
                    next: q,
                    current: Some(item.clone()),
                    vol: self.sink.volume(),
                    speed: self.sink.speed(),
                    pos: self.sink.get_pos(),
                });
            }
            WorkerState::Idle => {
                let _ = self.action_tx.send(Action::InQueue {
                    next: Vec::default(),
                    current: None,
                    vol: self.sink.volume(),
                    speed: self.sink.speed(),
                    pos: self.sink.get_pos(),
                });
            }
        };
    }
    fn continue_stream(&mut self) {
        self.sink.play();
    }

    fn pause_stream(&mut self) {
        self.sink.pause();
    }
    fn play_from_url(&self, url: String) -> CancellationToken {
        let sink = self.sink.clone();
        let sink2 = self.sink.clone();
        let action_tx = self.action_tx.clone();
        let action_tx2 = self.action_tx.clone();
        let player_tx = self.player_tx.clone();
        let token = CancellationToken::new();
        let cloned_token = token.clone();

        tokio::task::spawn(async move {
            let reader =
                match StreamReader::get_reader(url.parse().unwrap(), action_tx.clone()).await {
                    Ok(r) => r,
                    Err(_) => return,
                };
            let stream_token = reader.cancellation_token();
            let play = tokio::task::spawn_blocking(move || -> Result<(), StreamError> {
                let source = rodio::Decoder::new(reader).map_err(|e| StreamError::decode(e))?;
                sink.append(source);
                sink.sleep_until_end();
                Ok(())
            });
            let poll_state = tokio::task::spawn(async move {
                loop {
                    let _ = action_tx2.send(Action::PlayerState(
                        crate::action::StateType::Position(sink2.get_pos()),
                    ));
                    sleep(Duration::from_millis(500)).await;
                }
            });
            select! {
                _ = cloned_token.cancelled() => {
                    stream_token.cancel();
                    let _ = action_tx.send(Action::PlayerMessage("Stream cancelled by user.".to_string()));
                    // Player does not need to do anything more, as cancellation
                    // happens only when the stream is stopped or skipped
                }
                result = play => {
                    match result {
                        Ok(_) => {
                            // let _ = action_tx.send(Action::NowPlaying);
                        }

                        Err(e) => {
                            let _ = action_tx.send(Action::PlayerError(e.to_string()));
                        }
                    }
                    let _ = player_tx.send(PlayerAction::Skip);
                }
                _ = poll_state => {
                    let _ = action_tx.send(Action::PlayerError("Stream polling crashed?".to_string()));
                }
            }
        });
        token
    }
    fn skip(&mut self) {
        match &self.state {
            WorkerState::Playing { token, item: _ } => {
                self.sink.stop();
                token.cancel();
            }
            WorkerState::Loading { item: _ } => {
                self.sink.stop();
            }
            WorkerState::Idle => {}
        };
        match self.queue.pop_front() {
            Some(i) => {
                let _ = self
                    .action_tx
                    .send(Action::Query(Query::GetUrlByMedia { media: i.clone() }));
                self.state = WorkerState::Loading { item: i };
            }
            None => {
                self.state = WorkerState::Idle;
            }
        };
        self.send_playlist_state();
    }
    pub async fn run(&mut self) -> Result<()> {
        trace_dbg!("Starting PlayerWorker...");
        loop {
            let Some(event) = self.player_rx.recv().await else {
                break;
            };
            match event {
                PlayerAction::Stop => todo!(),
                PlayerAction::Pause => self.pause_stream(),
                PlayerAction::Continue => self.continue_stream(),
                PlayerAction::Kill => self.should_quit = true,
                PlayerAction::Skip => self.skip(),
                PlayerAction::AddToQueue { music, pos } => {
                    // TODO: Change add location based on pos
                    let was_empty = self.queue.is_empty();
                    match pos {
                        QueueLocation::Front => {
                            match &self.state {
                                WorkerState::Playing { token: _, item }
                                | WorkerState::Loading { item } => {
                                    // If music was currently being played, then add it back to the
                                    // queue
                                    self.queue.push_front(item.clone());
                                }
                                WorkerState::Idle => {}
                            }
                            music.into_iter().for_each(|m| self.queue.push_front(m));
                            self.skip();
                        }
                        QueueLocation::Next => {
                            music.into_iter().for_each(|m| self.queue.push_front(m))
                        }
                        QueueLocation::Last => {
                            music.into_iter().for_each(|m| self.queue.push_back(m))
                        }
                    }
                    if let WorkerState::Idle = self.state {
                        // If the queue was empty, then adding an item automatically starts playing
                        if was_empty {
                            self.skip();
                        }
                    }
                }
                PlayerAction::PlayURL { music, url } => {
                    // Ensure the one we get is what we expected
                    if let WorkerState::Loading { item } = &self.state {
                        if item.id == music.id {
                            let _ = self
                                .action_tx
                                .send(Action::PlayerMessage("Starting...".to_string()));
                            let token = self.play_from_url(url);
                            self.state = WorkerState::Playing { token, item: music };
                        };
                    };
                }
            };
            self.send_playlist_state();
            if self.should_quit {
                break;
            }
        }
        Ok(())
    }
}

impl PlayerWorker {
    pub fn new(
        handle: OutputStreamHandle,
        sender: UnboundedSender<Action>,
        config: Config,
    ) -> Self {
        let (player_tx, player_rx) = mpsc::unbounded_channel();
        let sink = Arc::from(rodio::Sink::try_new(&handle).unwrap());
        Self {
            player_tx,
            player_rx,
            action_tx: sender,
            should_quit: false,
            config,
            sink,
            state: WorkerState::Idle,
            queue: VecDeque::new(),
        }
    }
    pub fn get_tx(&self) -> UnboundedSender<PlayerAction> {
        self.player_tx.clone()
    }
}

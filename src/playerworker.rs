pub mod player;
pub mod streamerror;
pub mod streamreader;

use std::sync::Arc;
use std::time::Duration;

use color_eyre::Result;
use player::{PlayerAction, QueueLocation};
use rodio::{OutputStreamHandle, Sink};
use streamerror::StreamError;
use streamreader::StreamReader;
use tokio::select;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::action::getplaylist::Media;
use crate::action::{Action, PlayState};
use crate::config::Config;
use crate::queryworker::query::Query;
use crate::trace_dbg;

enum WorkerState {
    // The fetched file is played
    Playing {
        token: CancellationToken,
        item: Media,
        // This field exists for all three states, but is in this worker state to differentiate its
        // meaning from WorkerState::Idle
        current: usize,
    },
    // The file URL is being fetched
    Loading {
        item: Media,
        current: usize,
    },
    // There are no items at the play_next index
    Idle {
        // Index of the music currently in play, or music that should be played next
        // This is different because this may be out of bounds when the streaming have stopped
        // because the last song has been played.
        // If there three items [0, 1, 2], and the user listens to all 3, then `play_next` becomes
        // 3, which is not a valid index
        play_next: usize,
    },
}

pub struct PlayerWorker {
    state: WorkerState,
    player_tx: UnboundedSender<PlayerAction>,
    player_rx: UnboundedReceiver<PlayerAction>,
    action_tx: UnboundedSender<Action>,
    should_quit: bool,
    config: Config,
    sink: Arc<Sink>,
    queue: Vec<Media>,
}

impl PlayerWorker {
    fn add_musics(&mut self, items: Vec<Media>, pos: QueueLocation) {
        match pos {
            QueueLocation::Front => {
                match self.state {
                    WorkerState::Playing {
                        token: _,
                        item: _,
                        current,
                    }
                    | WorkerState::Loading { item: _, current } => {
                        self.queue.splice(current..current, items);
                        // The music being played right now is being modified
                        self.skip(0);
                    }
                    WorkerState::Idle { play_next } => {
                        self.queue.splice(play_next..play_next, items);
                    }
                };
            }
            QueueLocation::Next => {
                match self.state {
                    WorkerState::Playing {
                        token: _,
                        item: _,
                        current,
                    }
                    | WorkerState::Loading { item: _, current } => {
                        self.queue.splice((current + 1)..(current + 1), items);
                    }
                    WorkerState::Idle { play_next } => {
                        if play_next == self.queue.len() {
                            self.queue.append(&mut items.clone());
                        } else {
                            self.queue.splice((play_next + 1)..(play_next + 1), items);
                        };
                    }
                };
            }
            QueueLocation::Last => {
                self.queue.append(&mut items.clone());
            }
        };
    }
    fn send_playlist_state(&mut self) {
        let q = self.queue.clone().into();
        let action = match &self.state {
            WorkerState::Playing {
                token: _,
                item,
                current,
            } => Action::InQueue {
                play: PlayState::new(Some(item.clone()), q, *current),
                vol: self.sink.volume(),
                speed: self.sink.speed(),
                pos: self.sink.get_pos(),
            },
            WorkerState::Loading { item, current } => Action::InQueue {
                play: PlayState::new(Some(item.clone()), q, *current),
                vol: self.sink.volume(),
                speed: self.sink.speed(),
                pos: Duration::from_secs(0),
            },
            WorkerState::Idle { play_next } => Action::InQueue {
                play: PlayState::new(None, q, *play_next),
                vol: self.sink.volume(),
                speed: self.sink.speed(),
                pos: Duration::from_secs(0),
            },
        };
        let _ = self.action_tx.send(action);
    }
    fn continue_stream(&mut self) {
        self.sink.play();
    }

    fn pause_stream(&mut self) {
        self.sink.pause();
    }

    /// Given a URL, `play_from_url` fetches music file from this URL and plays it
    /// This spawns the following threads:
    /// Function
    /// |- Main playing thread
    ///    |- Polling thread
    ///
    /// Main playing thread spawns a polling thread, and then runs `select!` macro to wait for the
    /// music stream to complete.
    ///
    /// Polling thread sends out current player position every 500 miliseconds. This is run in a
    /// loop, and should never complete.
    ///
    /// The `select!` macro waits for 3 different things:
    /// 1. Main playing thread
    ///    The music has finished. `Skip` Action is sent out.
    /// 2. Cancellation
    ///    User has request cancellation, which causes music to stop.
    ///    `Skip` is NOT sent out; This is handled by the `skip` function itself. Cancellation simply
    ///    cancels the stream.
    /// 3. Polling thread
    ///    This should never happen, and will panic instead.
    ///
    /// Calling this function returns immediately with a token that can cancel the main playing
    /// thread
    fn play_from_url(&self, url: String) -> CancellationToken {
        // Used by Main playing thread to append decoded source into it
        let sink = self.sink.clone();
        let action_tx = self.action_tx.clone();
        // Used by Polling thread
        let sink2 = self.sink.clone();
        let action_tx2 = self.action_tx.clone();

        let player_tx = self.player_tx.clone();
        // Cancellation token used for returning
        let token = CancellationToken::new();
        // Cancellation token to listen to cancellation
        let cloned_token = token.clone();
        tokio::task::spawn(async move {
            let reader =
                match StreamReader::get_reader(url.parse().unwrap(), action_tx.clone()).await {
                    Ok(r) => r,
                    Err(_) => return,
                };
            let stream_token = reader.cancellation_token();
            let play: JoinHandle<Result<(), StreamError>> =
                tokio::task::spawn_blocking(move || {
                    // Panic may happen because Symphonia decoder is not being used
                    // Without Symphonia decoder, the decoding routine may contain `unwrap`
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
                    let _ = action_tx.send(Action::PlayerError("Stream polling crashed! Restart recommended.".to_string()));
                }
            }
        });
        token
    }
    fn skip(&mut self, skip_by: i32) {
        // Get the index of the music to play next
        let index = match &self.state {
            WorkerState::Playing {
                token,
                item: _,
                current,
            } => {
                self.sink.stop();
                token.cancel();
                (*current as i32) + skip_by
            }
            WorkerState::Loading { item: _, current } => {
                self.sink.stop();
                (*current as i32) + skip_by
            }
            WorkerState::Idle { play_next } => (*play_next) as i32,
        };
        let cleaned = if index >= 0 { index as usize } else { 0 };
        match self.queue.get(cleaned) {
            Some(i) => {
                let _ = self
                    .action_tx
                    .send(Action::Query(Query::GetUrlByMedia { media: i.clone() }));
                self.state = WorkerState::Loading {
                    item: i.clone(),
                    current: cleaned,
                };
            }
            None => self.state = WorkerState::Idle { play_next: cleaned },
        }
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
                PlayerAction::Skip => self.skip(1),
                PlayerAction::AddToQueue { music, pos } => {
                    if self.queue.is_empty() {
                        self.queue = music;
                        if let WorkerState::Idle { play_next: _ } = self.state {
                            // If the queue was empty, then adding an item automatically starts playing
                            self.skip(0);
                        }
                    } else {
                        self.add_musics(music, pos);
                    };
                }
                PlayerAction::PlayURL { music, url } => {
                    // Ensure the one we get is what we expected
                    if let WorkerState::Loading { item, current } = &self.state {
                        if item.id == music.id {
                            let _ = self
                                .action_tx
                                .send(Action::PlayerMessage("Starting...".to_string()));
                            let token = self.play_from_url(url);
                            self.state = WorkerState::Playing {
                                token,
                                item: music,
                                current: *current,
                            };
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
            state: WorkerState::Idle { play_next: 0 },
            queue: Vec::new(),
        }
    }
    pub fn get_tx(&self) -> UnboundedSender<PlayerAction> {
        self.player_tx.clone()
    }
}

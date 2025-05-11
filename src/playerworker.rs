pub mod player;
pub mod streamerror;

use std::collections::VecDeque;
use std::sync::Arc;

use color_eyre::Result;
use player::PlayerAction;
use reqwest::Url;
use rodio::{OutputStreamHandle, Sink};
use stream_download::http::HttpStream;
use streamerror::StreamError;
use tokio::select;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;

use crate::action::Action;
use crate::config::Config;
use crate::trace_dbg;
use stream_download::source::SourceStream;
use stream_download::storage::temp::TempStorageProvider;
use stream_download::{Settings, StreamDownload};

pub struct QueueItem {
    url: String,
}

impl Clone for QueueItem {
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
        }
    }
}

enum WorkerState {
    // The queue item is queried, and the file is fetched from the server
    TryPlay {
        token: CancellationToken,
        item: QueueItem,
    },
    // The fetched file is played
    Playing(QueueItem),
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
    queue: VecDeque<QueueItem>,
}

impl PlayerWorker {
    fn continue_stream(&mut self) {
        self.sink.play();
    }

    fn pause_stream(&mut self) {
        self.sink.pause();
    }
    async fn load_from_url(
        url: String,
    ) -> Result<StreamDownload<TempStorageProvider>, StreamError> {
        let url = url.parse::<Url>().map_err(|_| StreamError::parse(url))?;
        let stream = HttpStream::<stream_download::http::reqwest::Client>::create(url)
            .await
            .map_err(|e| StreamError::stream(e))?;
        Ok(
            StreamDownload::from_stream(stream, TempStorageProvider::new(), Settings::default())
                .await
                .map_err(|e| StreamError::stream_init(e))?,
        )
    }
    async fn start_stream(sink: Arc<Sink>, url: String) -> Result<(), StreamError> {
        let reader = PlayerWorker::load_from_url(url).await?;
        tokio::task::spawn_blocking(move || {
            sink.append(rodio::Decoder::new(reader).unwrap());
            sink.sleep_until_end();
        });
        Ok(())
    }
    fn play_from_url(&mut self, url: String) {
        let c = url.clone();
        let sink = self.sink.clone();
        let action_tx = self.action_tx.clone();
        let player_tx = self.player_tx.clone();
        let token = CancellationToken::new();
        let cloned_token = token.clone();

        tokio::spawn(async move {
            select! {
                _ = cloned_token.cancelled() => {
                    let _ = action_tx.send(Action::StreamError("Stream cancelled by user.".to_string()));
                    // Player does not need to do anything more, as cancellation
                    // happens only when the stream is stopped or skipped
                }
                result = PlayerWorker::start_stream(sink, url) => {
                    match result {
                        Ok(_) => {
                            let _ = action_tx.send(Action::NowPlaying);
                            let _ = player_tx.send(PlayerAction::Playing);
                        }

                        Err(e) => {
                            let _ = action_tx.send(Action::StreamError(e.to_string()));
                            let _ = player_tx.send(PlayerAction::Skip);
                        }
                    }
                }
            }
        });

        self.state = WorkerState::TryPlay {
            token,
            item: QueueItem { url: c },
        };
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
                PlayerAction::TryPlay { url } => self.play_from_url(url),
                PlayerAction::Continue => self.continue_stream(),
                PlayerAction::Kill => self.should_quit = true,
                PlayerAction::Playing => {
                    if let WorkerState::TryPlay { token, item } = &self.state {
                        self.state = WorkerState::Playing(item.clone());
                    } else {
                        let _ = self.action_tx.send(Action::PlayerError(
                            "Invalid state transition to Playing.".to_string(),
                        ));
                    }
                }
                PlayerAction::Skip => {
                    match self.queue.pop_front() {
                        Some(item) => {
                            // If the player was in the process of fetching an item, cancel it
                            if let WorkerState::TryPlay { token, item: _ } = &self.state {
                                token.cancel();
                            }
                            self.sink.stop();
                        }
                        // If the queue is empty, then skip should put the player into idle mode.
                        None => self.state = WorkerState::Idle,
                    };
                }
            };
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

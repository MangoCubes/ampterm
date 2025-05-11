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
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

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
    TryPlay(QueueItem),
    Playing(QueueItem),
    Stopped,
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

// Three actions are available to adding items to the queue
// 1. Add next
// 2. Play now
// 3. Add last

impl PlayerWorker {
    fn continue_stream(&mut self) {
        self.sink.play();
    }

    fn pause_stream(&mut self) {
        self.sink.pause();
    }
    async fn start_stream(sink: Arc<Sink>, url: String) -> Result<(), StreamError> {
        let url = url.parse::<Url>().map_err(|_| StreamError::parse(url))?;
        let stream = HttpStream::<stream_download::http::reqwest::Client>::create(url)
            .await
            .map_err(|e| StreamError::stream(e))?;
        let reader =
            StreamDownload::from_stream(stream, TempStorageProvider::new(), Settings::default())
                .await
                .map_err(|e| StreamError::stream_init(e))?;
        tokio::task::spawn_blocking(move || {
            sink.append(rodio::Decoder::new(reader).unwrap());
            sink.sleep_until_end();
        });
        Ok(())
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
                PlayerAction::TryPlay { url } => {
                    let c = url.clone();
                    let sink = self.sink.clone();
                    let action_tx = self.action_tx.clone();
                    let player_tx = self.player_tx.clone();

                    tokio::spawn(async move {
                        match PlayerWorker::start_stream(sink, url).await {
                            Ok(_) => {
                                let _ = action_tx.send(Action::NowPlaying);
                                let _ = player_tx.send(PlayerAction::Playing);
                            }

                            Err(e) => {
                                let _ = action_tx.send(Action::StreamError(e.to_string()));
                                let _ = player_tx.send(PlayerAction::Cancel);
                            }
                        }
                    });

                    self.state = WorkerState::TryPlay(QueueItem { url: c });
                }
                PlayerAction::Continue => self.continue_stream(),
                PlayerAction::Kill => self.should_quit = true,
                PlayerAction::Playing => {
                    if let WorkerState::TryPlay(s) = &self.state {
                        self.state = WorkerState::Playing(s.clone());
                    } else {
                        let _ = self.action_tx.send(Action::PlayerError(
                            "Stream was cancelled whilst fetching.".to_string(),
                        ));
                    }
                }
                PlayerAction::Cancel => {
                    self.state = WorkerState::Stopped;
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
            state: WorkerState::Stopped,
            queue: VecDeque::new(),
        }
    }
    pub fn get_tx(&self) -> UnboundedSender<PlayerAction> {
        self.player_tx.clone()
    }
}

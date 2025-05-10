pub mod player;
pub mod streamerror;

use std::sync::Arc;

use color_eyre::{eyre, Result};
use player::PlayerAction;
use reqwest::{Client, Url};
use rodio::{OutputStream, OutputStreamHandle, Sink};
use stream_download::http::HttpStream;
use streamerror::StreamError;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;

use crate::action::Action;
use crate::config::Config;
use crate::trace_dbg;
use stream_download::source::SourceStream;
use stream_download::storage::temp::TempStorageProvider;
use stream_download::{Settings, StreamDownload};

enum WorkerState {
    Playing {
        paused: bool,
        token: CancellationToken,
    },
    Stopped,
}

struct OutputDevice {
    stream: OutputStream,
    handle: OutputStreamHandle,
}

pub struct PlayerWorker {
    state: WorkerState,
    player_tx: UnboundedSender<PlayerAction>,
    player_rx: UnboundedReceiver<PlayerAction>,
    action_tx: UnboundedSender<Action>,
    should_quit: bool,
    config: Config,
    handle: Arc<OutputStreamHandle>,
}

impl PlayerWorker {
    fn continue_stream(&mut self) -> Result<(), StreamError> {
        if let WorkerState::Playing { paused, token: _ } = &mut self.state {
            *paused = false;
        }
        Ok(())
    }

    fn pause_stream(&mut self) -> Result<(), StreamError> {
        if let WorkerState::Playing { paused, token: _ } = &mut self.state {
            *paused = true;
        }
        Ok(())
    }
    async fn start_stream(handle: Arc<OutputStreamHandle>, url: String) -> Result<(), StreamError> {
        let url = url.parse::<Url>().map_err(|_| StreamError::parse(url))?;
        let stream = HttpStream::<stream_download::http::reqwest::Client>::create(url)
            .await
            .map_err(|e| StreamError::stream(e))?;
        let reader =
            StreamDownload::from_stream(stream, TempStorageProvider::new(), Settings::default())
                .await
                .map_err(|e| StreamError::stream_init(e))?;
        let sink = Arc::from(rodio::Sink::try_new(&handle).unwrap());
        let handle = tokio::task::spawn_blocking(move || {
            sink.append(rodio::Decoder::new(reader).unwrap());
            sink.sleep_until_end();
        });
        handle.await;
        Ok(())
    }
    pub async fn run(&mut self) -> Result<()> {
        trace_dbg!("Starting PlayerWorker...");
        loop {
            let Some(event) = self.player_rx.recv().await else {
                break;
            };
            let a = match event {
                PlayerAction::Stop => todo!(),
                PlayerAction::Pause => self.pause_stream(),
                PlayerAction::Play { url } => {
                    let token = CancellationToken::new();
                    let token_clone = token.clone();

                    self.state = WorkerState::Playing {
                        token: token_clone,
                        paused: false,
                    };

                    let handle = self.handle.clone();

                    tokio::spawn(async move { PlayerWorker::start_stream(handle, url).await });
                    Ok(())
                }
                PlayerAction::Continue => self.continue_stream(),
                PlayerAction::Kill => todo!(),
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
        Self {
            player_tx,
            player_rx,
            action_tx: sender,
            should_quit: false,
            config,
            handle: Arc::from(handle),
            state: WorkerState::Stopped,
        }
    }
    pub fn get_tx(&self) -> UnboundedSender<PlayerAction> {
        self.player_tx.clone()
    }
}

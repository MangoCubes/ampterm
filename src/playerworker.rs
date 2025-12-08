pub mod player;
mod streamerror;
mod streamreader;

use std::sync::Arc;
use std::time::Duration;

use color_eyre::Result;
use player::ToPlayerWorker;
use rodio::{OutputStreamHandle, Sink};
use streamerror::StreamError;
use streamreader::StreamReader;
use tokio::select;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::action::action::{Action, QueryAction};
use crate::config::Config;
use crate::playerworker::player::{FromPlayerWorker, StateType};
use crate::trace_dbg;

enum WorkerState {
    // The fetched file is played
    Playing(CancellationToken),
    Idle,
}

pub struct PlayerWorker {
    state: WorkerState,
    player_tx: UnboundedSender<ToPlayerWorker>,
    player_rx: UnboundedReceiver<ToPlayerWorker>,
    action_tx: UnboundedSender<Action>,
    should_quit: bool,
    sink: Arc<Sink>,
    last_speed_change: Duration,
    /// Total accumulated length of a song played
    /// [`total_length / played_in`] gives us the average speed
    total_length: Duration,
    /// Actual runtime
    played_in: Duration,
}

impl PlayerWorker {
    fn continue_stream(&mut self) {
        self.sink.play();
    }

    fn pause_stream(&mut self) {
        self.sink.pause();
    }

    fn avg_speed(&mut self) -> f32 {
        let pos = self.sink.get_pos();
        let seg = pos - self.last_speed_change;
        self.total_length += seg.mul_f32(self.sink.speed());
        self.played_in += seg;
        self.last_speed_change = pos;
        self.total_length.div_duration_f32(self.played_in)
    }

    fn reset_avg_speed(&mut self) {
        self.total_length = Duration::from_secs(0);
        self.played_in = Duration::from_secs(0);
        self.last_speed_change = Duration::from_secs(0);
    }

    fn jump(&mut self, offset: f32) -> Duration {
        // Get average speed throughout the entire play
        let avg = self.avg_speed();
        // Get current speed
        let orig = self.sink.speed();
        // Get current REAL position by multiplying by average speed
        let pos = self.sink.get_pos().mul_f32(avg);
        // Get REAL absolute offset
        let target = if offset >= 0.0 {
            let real_offset = Duration::from_secs_f32(offset);
            pos + real_offset
        } else {
            let real_offset = Duration::from_secs_f32(-offset);
            if real_offset >= pos {
                Duration::from_secs(0)
            } else {
                pos - real_offset
            }
        };
        // Convert to internal timer by dividing by current speed
        let ret = target.div_f32(orig);
        self.sink.try_seek(ret);
        self.reset_avg_speed();
        ret
    }

    fn change_speed(&mut self, to: f32) {
        let pos = self.sink.get_pos();
        let seg = pos - self.last_speed_change;
        self.total_length += seg.mul_f32(self.sink.speed());
        self.played_in += seg;
        self.last_speed_change = pos;
        self.sink.set_speed(to);
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
    /// Polling thread sends out current player position every 100 miliseconds. This is run in a
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
                    let _ = action_tx2.send(Action::Query(QueryAction::FromPlayerWorker(
                        FromPlayerWorker::StateChange(StateType::Position(sink2.get_pos())),
                    )));
                    sleep(Duration::from_millis(100)).await;
                }
            });
            select! {
                _ = cloned_token.cancelled() => {
                    stream_token.cancel();
                    let _ = action_tx.send(Action::Query(QueryAction::FromPlayerWorker(
                                FromPlayerWorker::Message("Stream cancelled by user.".to_string()),
                    )));
                    // Player does not need to do anything more, as cancellation
                    // happens only when the stream is stopped or skipped
                }
                result = play => {
                    match result {
                        Ok(_) => {
                            // let _ = action_tx.send(Action::NowPlaying);
                        }

                        Err(e) => {
                            let _ = action_tx.send(Action::Query(QueryAction::FromPlayerWorker(
                                        FromPlayerWorker::Error(e.to_string()),
                            )));
                        }
                    }
                    // Regardless of the error occurred, move on
                    let _ = action_tx.send(Action::Query(QueryAction::FromPlayerWorker(FromPlayerWorker::Finished)));
                }
                _ = poll_state => {
                    // let _ = action_tx.send(Action::PlayerError("Stream polling crashed! Restart recommended.".to_string()));
                }
            }
        });
        token
    }

    #[inline]
    fn send_action(&self, action: FromPlayerWorker) {
        let _ = self
            .action_tx
            .send(Action::Query(QueryAction::FromPlayerWorker(action)));
    }

    pub async fn run(&mut self) -> Result<()> {
        trace_dbg!("Starting PlayerWorker...");
        loop {
            let Some(event) = self.player_rx.recv().await else {
                break;
            };
            match event {
                ToPlayerWorker::Stop => {
                    self.sink.stop();
                    if let WorkerState::Playing(token) = &self.state {
                        token.cancel();
                    };
                    self.send_action(FromPlayerWorker::StateChange(StateType::NowPlaying(None)));
                }
                ToPlayerWorker::Pause => self.pause_stream(),
                ToPlayerWorker::Resume => self.continue_stream(),
                ToPlayerWorker::Kill => self.should_quit = true,
                ToPlayerWorker::PlayURL { music, url } => {
                    self.sink.stop();
                    if let WorkerState::Playing(token) = &self.state {
                        token.cancel();
                    };
                    self.send_action(FromPlayerWorker::StateChange(StateType::NowPlaying(Some(
                        music,
                    ))));
                    let token = self.play_from_url(url);
                    self.state = WorkerState::Playing(token);
                }
                ToPlayerWorker::GoToStart => {
                    if let Err(e) = self.sink.try_seek(Duration::from_secs(0)) {
                        self.send_action(FromPlayerWorker::Message(format!(
                            "Failed to seek: {}",
                            e
                        )));
                    } else {
                        self.send_action(FromPlayerWorker::StateChange(StateType::Jump(
                            Duration::from_secs(0),
                        )));
                    }
                }
                ToPlayerWorker::ChangeSpeed(by) => {
                    let current = self.sink.speed();
                    let new_speed = current + by;
                    let cleaned = if new_speed <= 0.0 { 0.01 } else { new_speed };
                    self.change_speed(cleaned);
                    self.send_action(FromPlayerWorker::StateChange(StateType::Speed(cleaned)));
                }
                ToPlayerWorker::SetSpeed(to) => {
                    let cleaned = if to <= 0.0 { 0.01 } else { to };
                    self.change_speed(cleaned);
                    self.send_action(FromPlayerWorker::StateChange(StateType::Speed(cleaned)));
                }
                ToPlayerWorker::ChangeVolume(by) => {
                    let current = self.sink.volume();
                    let new_vol = current + by;
                    let cleaned = if new_vol < 0.0 {
                        0.0
                    } else if new_vol > 1.0 {
                        1.0
                    } else {
                        new_vol
                    };
                    self.sink.set_volume(cleaned);
                    self.send_action(FromPlayerWorker::StateChange(StateType::Volume(cleaned)));
                }
                ToPlayerWorker::SetVolume(to) => {
                    let cleaned = if to < 0.0 {
                        0.0
                    } else if to > 1.0 {
                        1.0
                    } else {
                        to
                    };
                    self.sink.set_volume(cleaned);
                    self.send_action(FromPlayerWorker::StateChange(StateType::Volume(cleaned)));
                }
                ToPlayerWorker::ResumeOrPause => {
                    if self.sink.is_paused() {
                        self.continue_stream();
                    } else {
                        self.pause_stream();
                    }
                }
                ToPlayerWorker::ChangePosition(by) => {
                    let newpos = self.jump(by);
                    self.send_action(FromPlayerWorker::StateChange(StateType::Jump(newpos)));
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
        sink.set_volume(config.init_state.volume);
        Self {
            player_tx,
            player_rx,
            action_tx: sender,
            should_quit: false,
            sink,
            state: WorkerState::Idle,
            last_speed_change: Duration::from_secs(0),
            total_length: Duration::from_secs(0),
            played_in: Duration::from_secs(0),
        }
    }
    pub fn get_tx(&self) -> UnboundedSender<ToPlayerWorker> {
        self.player_tx.clone()
    }
}

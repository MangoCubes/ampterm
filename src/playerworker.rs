pub mod player;
pub mod playerstatus;
mod realtime;
mod streamerror;
mod streamreader;

use std::sync::Arc;
use std::time::Duration;

use color_eyre::Result;
use player::ToPlayerWorker;
use rodio::{OutputStream, Sink};
use streamerror::StreamError;
use streamreader::StreamReader;
use tokio::select;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::action::action::{Action, TargetedAction};
use crate::config::Config;
use crate::playerworker::player::FromPlayerWorker;
use crate::playerworker::playerstatus::PlayerStatus;
use crate::playerworker::realtime::{PosTime, RealTime};
use crate::queryworker::highlevelquery::HighLevelQuery;
use crate::queryworker::query::ToQueryWorker;
use crate::trace_dbg;

enum WorkerState {
    // The fetched file is played
    Playing(CancellationToken),
    Idle,
}

pub struct PlayerWorker {
    state: WorkerState,
    playerstatus: Arc<RwLock<PlayerStatus>>,
    player_tx: UnboundedSender<ToPlayerWorker>,
    player_rx: UnboundedReceiver<ToPlayerWorker>,
    action_tx: UnboundedSender<Action>,
    should_quit: bool,
    sink: Arc<Sink>,
    timer: RealTime,
    handle: OutputStream,
}

impl PlayerWorker {
    async fn continue_stream(&mut self) {
        self.sink.play();
        let mut lock = self.playerstatus.write().await;
        lock.playing = true;
        self.send_player_msg(FromPlayerWorker::Playing(true));
    }

    async fn pause_stream(&mut self) {
        self.sink.pause();
        let mut lock = self.playerstatus.write().await;
        lock.playing = false;
        self.send_player_msg(FromPlayerWorker::Playing(false));
    }

    fn jump_to(&mut self, to: f32) -> Duration {
        let (newpos, sink_pos) = self
            .timer
            .move_time_to(PosTime::from_secs_f32(to), self.sink.speed());
        if let Err(e) = self.sink.try_seek(sink_pos) {
            self.send_err(format!("Seeking failed! Reason: {e}"));
        };
        newpos
    }

    fn jump(&mut self, offset: f32) -> Duration {
        let (newpos, sink_pos) =
            self.timer
                .move_time_by(self.sink.get_pos(), self.sink.speed(), offset);
        if let Err(e) = self.sink.try_seek(sink_pos) {
            self.send_err(format!("Seeking failed! Reason: {e}"));
        };
        newpos
    }

    fn change_speed(&mut self, to: f32) {
        self.timer.add(self.sink.get_pos(), self.sink.speed());
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
                    let len = reader.content_length();
                    let builder = rodio::Decoder::builder().with_data(reader);
                    let source = match len {
                        Some(len) => builder.with_byte_len(len),
                        None => builder,
                    }
                    .build()
                    .map_err(|e| StreamError::decode(e))?;

                    sink.append(source);
                    sink.sleep_until_end();
                    Ok(())
                });
            let poll_state = tokio::task::spawn(async move {
                loop {
                    let _ = player_tx.send(ToPlayerWorker::Tick);
                    sleep(Duration::from_millis(200)).await;
                }
            });
            select! {
                _ = cloned_token.cancelled() => {
                    stream_token.cancel();
                    let _ = action_tx.send(Action::Targeted(TargetedAction::Info(
                                "Stream cancelled by user.".to_string()
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
                            let _ = action_tx.send(Action::Targeted(TargetedAction::Err(
                                        e.to_string()
                            )));
                        }
                    }
                    // Regardless of the error occurred, move on
                    let _ = action_tx.send(Action::FromPlayer(FromPlayerWorker::Finished));
                }
                _ = poll_state => {
                    // let _ = action_tx.send(Action::PlayerError("Stream polling crashed! Restart recommended.".to_string()));
                }
            }
        });
        token
    }

    #[inline]
    fn send_info(&self, msg: String) {
        let _ = self
            .action_tx
            .send(Action::Targeted(TargetedAction::Info(msg)));
    }
    #[inline]
    fn send_err(&self, msg: String) {
        let _ = self
            .action_tx
            .send(Action::Targeted(TargetedAction::Err(msg)));
    }

    #[inline]
    fn send_player_msg(&self, action: FromPlayerWorker) {
        let _ = self.action_tx.send(Action::FromPlayer(action));
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
                    self.timer.reset();
                    let mut lock = self.playerstatus.write().await;
                    lock.now_playing = None;
                    self.send_player_msg(FromPlayerWorker::NowPlaying(None));
                }
                ToPlayerWorker::Pause => self.pause_stream().await,
                ToPlayerWorker::Resume => self.continue_stream().await,
                ToPlayerWorker::Kill => self.should_quit = true,
                ToPlayerWorker::PlayMedia { media } => {
                    let _ = self.action_tx.send(Action::ToQuery(ToQueryWorker::new(
                        HighLevelQuery::PlayMusicFromURL(media.clone()),
                    )));
                }
                ToPlayerWorker::PlayURL { music, url } => {
                    self.sink.stop();
                    if let WorkerState::Playing(token) = &self.state {
                        token.cancel();
                    };
                    let mut lock = self.playerstatus.write().await;
                    lock.now_playing = Some(music.clone());
                    lock.playing = true;
                    let token = self.play_from_url(url);
                    self.send_player_msg(FromPlayerWorker::NowPlaying(Some(music)));
                    self.send_player_msg(FromPlayerWorker::Playing(true));
                    self.timer.reset();
                    self.state = WorkerState::Playing(token);
                }
                ToPlayerWorker::GoToStart => {
                    if let Err(e) = self.sink.try_seek(Duration::from_secs(0)) {
                        self.send_err(format!("Failed to seek: {}", e));
                    } else {
                        self.send_player_msg(FromPlayerWorker::Jump(Duration::from_secs(0)));
                    };
                    self.timer.reset();
                }
                ToPlayerWorker::ChangeSpeed(by) => {
                    let current = self.sink.speed();
                    let new_speed = current + by;
                    let cleaned = if new_speed <= 0.0 { 0.01 } else { new_speed };
                    self.change_speed(cleaned);
                    let mut lock = self.playerstatus.write().await;
                    lock.speed = cleaned;
                    self.send_player_msg(FromPlayerWorker::Speed(cleaned));
                }
                ToPlayerWorker::SetSpeed(to) => {
                    let cleaned = if to <= 0.0 { 0.01 } else { to };
                    self.change_speed(cleaned);
                    let mut lock = self.playerstatus.write().await;
                    lock.speed = cleaned;
                    self.send_player_msg(FromPlayerWorker::Speed(cleaned));
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
                    let mut lock = self.playerstatus.write().await;
                    lock.volume = cleaned;
                    self.send_player_msg(FromPlayerWorker::Volume(cleaned));
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
                    let mut lock = self.playerstatus.write().await;
                    lock.volume = cleaned;
                    self.send_player_msg(FromPlayerWorker::Volume(cleaned));
                }
                ToPlayerWorker::ResumeOrPause => {
                    if self.sink.is_paused() {
                        self.continue_stream().await;
                    } else {
                        self.pause_stream().await;
                    }
                }
                ToPlayerWorker::ChangePosition(by) => {
                    let newpos = self.jump(by);
                    self.send_player_msg(FromPlayerWorker::Jump(newpos));
                }
                ToPlayerWorker::SetPosition(to) => {
                    let newpos = self.jump_to(to);
                    self.send_player_msg(FromPlayerWorker::Jump(newpos));
                }
                ToPlayerWorker::Tick => {
                    // There is a bug with rodio that happens when a new music is loaded. The
                    // reported position [`get_pos()`] falsely reports the end of the last media
                    // played instead of the current one. It seems that the position is corrected
                    // after the new music is loaded. As a result, position reports that happen in
                    // this gap is equal to the position in the last media when the media is
                    // swapped.
                    // This issue is addressed by [`self.timer`], where if the position somehow
                    // goes backwards, it is blindly trusted.
                    self.timer.add(self.sink.get_pos(), self.sink.speed());
                    let pos = self.timer.get_now();
                    let mut lock = self.playerstatus.write().await;
                    lock.position = pos;
                    self.send_player_msg(FromPlayerWorker::Position(pos));
                }
            };
            if self.should_quit {
                break;
            }
        }
        self.handle.log_on_drop(false);
        Ok(())
    }
}

impl PlayerWorker {
    pub fn new(
        playerstatus: Arc<RwLock<PlayerStatus>>,
        handle: OutputStream,
        sender: UnboundedSender<Action>,
        config: Config,
    ) -> Self {
        let (player_tx, player_rx) = mpsc::unbounded_channel();
        let sink = Arc::from(rodio::Sink::connect_new(handle.mixer()));
        sink.set_volume(config.init_state.volume);
        Self {
            playerstatus,
            handle,
            player_tx,
            player_rx,
            action_tx: sender,
            should_quit: false,
            sink,
            state: WorkerState::Idle,
            timer: RealTime::new(),
        }
    }
    pub fn get_tx(&self) -> UnboundedSender<ToPlayerWorker> {
        self.player_tx.clone()
    }
}

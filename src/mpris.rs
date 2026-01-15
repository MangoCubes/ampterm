use std::sync::Arc;

use mpris_server::{
    zbus::{fdo, Result},
    LoopStatus, Metadata, PlaybackRate, PlaybackStatus, PlayerInterface, Property, RootInterface,
    Server, Time, TrackId, Volume,
};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    RwLock,
};

use crate::{
    action::action::{Action, TargetedAction},
    osclient::response::getplaylist::Media,
    playerworker::{player::FromPlayerWorker, playerstatus::PlayerStatus},
};

pub struct AmptermMpris {
    action_tx: UnboundedSender<Action>,
    playerstatus: Arc<RwLock<PlayerStatus>>,
}

impl RootInterface for AmptermMpris {
    async fn quit(&self) -> fdo::Result<()> {
        self.send(TargetedAction::Quit);
        Ok(())
    }

    async fn can_quit(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn set_fullscreen(&self, _fullscreen: bool) -> Result<()> {
        Ok(())
    }

    async fn can_set_fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }
    async fn raise(&self) -> fdo::Result<()> {
        Ok(())
    }

    async fn can_raise(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn has_track_list(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn identity(&self) -> fdo::Result<String> {
        Ok("Ampterm".to_string())
    }

    async fn desktop_entry(&self) -> fdo::Result<String> {
        Ok("ampterm".to_string())
    }

    async fn supported_uri_schemes(&self) -> fdo::Result<Vec<String>> {
        Ok(vec![])
    }

    async fn supported_mime_types(&self) -> fdo::Result<Vec<String>> {
        Ok(vec![])
    }
}

impl AmptermMpris {
    async fn get_media_metadata(media: &Option<Media>) -> Metadata {
        let metadata = Metadata::builder();
        if let Some(media) = &media {
            metadata
                .artist([media.artist.clone().unwrap_or("Unknown Artist".to_string())])
                .album(media.album.clone().unwrap_or("Unknown Album".to_string()))
                .title(media.title.clone())
                .length(Time::from_secs(media.duration.unwrap_or(0) as i64))
                .build()
        } else {
            metadata.title("Nothing in queue").build()
        }
    }
    fn send(&self, ta: TargetedAction) {
        let _ = self.action_tx.send(Action::Targeted(ta));
    }

    #[allow(dead_code)]
    pub fn new(
        action_tx: UnboundedSender<Action>,
        playerstatus: Arc<RwLock<PlayerStatus>>,
    ) -> Self {
        Self {
            action_tx,
            playerstatus,
        }
    }
    #[allow(dead_code)]
    pub async fn run(
        &self,
        mut mpris_rx: UnboundedReceiver<FromPlayerWorker>,
        server: &Server<AmptermMpris>,
    ) {
        loop {
            while let Some(ev) = mpris_rx.recv().await {
                match ev {
                    FromPlayerWorker::Playing(b) => {
                        let _ = server
                            .properties_changed([Property::PlaybackStatus(if b {
                                PlaybackStatus::Playing
                            } else {
                                PlaybackStatus::Paused
                            })])
                            .await;
                    }
                    FromPlayerWorker::NowPlaying(media) => {
                        let _ = server
                            .properties_changed([Property::Metadata(
                                Self::get_media_metadata(&media).await,
                            )])
                            .await;
                    }
                    FromPlayerWorker::Volume(v) => {
                        let _ = server
                            .properties_changed([Property::Volume(v.into())])
                            .await;
                    }
                    FromPlayerWorker::Speed(s) => {
                        let _ = server.properties_changed([Property::Rate(s.into())]).await;
                    }
                    _ => {}
                }
            }
        }
    }
}

impl PlayerInterface for AmptermMpris {
    async fn next(&self) -> fdo::Result<()> {
        self.send(TargetedAction::Skip);
        Ok(())
    }

    async fn previous(&self) -> fdo::Result<()> {
        self.send(TargetedAction::Previous);
        Ok(())
    }

    async fn pause(&self) -> fdo::Result<()> {
        self.send(TargetedAction::Pause);
        Ok(())
    }

    async fn play_pause(&self) -> fdo::Result<()> {
        self.send(TargetedAction::PlayOrPause);
        Ok(())
    }

    async fn stop(&self) -> fdo::Result<()> {
        self.send(TargetedAction::Stop); // For now
        Ok(())
    }

    async fn play(&self) -> fdo::Result<()> {
        self.send(TargetedAction::Play);
        Ok(())
    }

    async fn seek(&self, offset: Time) -> fdo::Result<()> {
        self.send(TargetedAction::ChangePosition(offset.as_secs() as f32));
        Ok(())
    }

    async fn set_position(&self, _track_id: TrackId, _position: Time) -> fdo::Result<()> {
        Ok(())
    }

    async fn open_uri(&self, _uri: String) -> fdo::Result<()> {
        Ok(())
    }

    async fn playback_status(&self) -> fdo::Result<PlaybackStatus> {
        let lock = self.playerstatus.read().await;
        Ok(if lock.playing {
            PlaybackStatus::Playing
        } else {
            PlaybackStatus::Paused
        })
    }

    async fn loop_status(&self) -> fdo::Result<LoopStatus> {
        Ok(LoopStatus::None)
    }

    async fn set_loop_status(&self, _loop_status: LoopStatus) -> Result<()> {
        Ok(())
    }

    async fn rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(PlaybackRate::default())
    }

    async fn set_rate(&self, rate: PlaybackRate) -> Result<()> {
        self.send(TargetedAction::SetSpeed(rate as f32));
        Ok(())
    }

    async fn set_shuffle(&self, shuffle: bool) -> Result<()> {
        if shuffle {
            self.send(TargetedAction::Shuffle);
        }
        Ok(())
    }

    async fn metadata(&self) -> fdo::Result<Metadata> {
        let lock = self.playerstatus.read().await;
        Ok(Self::get_media_metadata(&lock.now_playing).await)
    }

    async fn volume(&self) -> fdo::Result<Volume> {
        Ok(Volume::default())
    }

    async fn set_volume(&self, volume: Volume) -> Result<()> {
        self.send(TargetedAction::SetVolume(volume as f32));
        Ok(())
    }

    async fn position(&self) -> fdo::Result<Time> {
        let lock = self.playerstatus.read().await;
        Ok(Time::from_millis(lock.position.as_millis() as i64))
    }

    async fn minimum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(0.01)
    }

    async fn maximum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(1000.0)
    }

    async fn can_go_next(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_go_previous(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_play(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_pause(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_seek(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_control(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn shuffle(&self) -> fdo::Result<bool> {
        Ok(false)
    }
}

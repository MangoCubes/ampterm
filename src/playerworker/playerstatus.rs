use std::time::Duration;

use crate::osclient::response::getplaylist::Media;

pub struct PlayerStatus {
    pub playing: bool,
    pub position: Duration,
    pub now_playing: Option<Media>,
    pub volume: f32,
    pub speed: f32,
}

impl Default for PlayerStatus {
    fn default() -> Self {
        Self {
            playing: false,
            position: Duration::default(),
            now_playing: None,
            volume: 0.5,
            speed: 1.0,
        }
    }
}

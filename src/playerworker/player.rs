use std::time::Duration;

use serde::{Deserialize, Serialize};
use strum::Display;

use crate::osclient::response::getplaylist::Media;

#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum QueueLocation {
    // Music is added before the first element in the queue
    // When this action is invoked, the current song is stopped, and the sent music is added
    Front,
    // Music is added after the first element in the queue
    Next,
    // Music is added at the end of the queue
    Last,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToPlayerWorker {
    /// Stop the player. Unlike pause, this resets the progress for the current song.
    Stop,
    /// Stops the player, and preserves the progress for the current song.
    Pause,
    /// Resumes the player.
    Resume,
    /// Pauses or resumes the player depending on the current playing state.
    ResumeOrPause,
    /// Go to the start of the current song.
    GoToStart,
    /// Change the volume. Volume is represented as a decimal value between 0~1, and the passed value is added to the current volume.
    /// To reduce the value, pass a negative number.
    ChangeVolume(f32),
    /// Sets the volume absolutely.
    SetVolume(f32),

    ChangeSpeed(f32),
    SetSpeed(f32),

    ChangePosition(f32),

    /// Below should not be used by the user directly.
    PlayURL {
        music: Media,
        url: String,
    },
    PlayMedia {
        media: Media,
    },
    Kill,
    Tick,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum PlayOrder {
    Normal,
    Random,
    Reverse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FromPlayerWorker {
    Playing(bool),
    Jump(Duration),
    Position(Duration),
    NowPlaying(Option<Media>),
    Volume(f32),
    Speed(f32),
    Finished,
}

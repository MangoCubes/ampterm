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
    Continue,
    /// Go to the previous song.
    Previous,
    /// Go to the start of the current song.
    GoToStart,
    /// Go to the next song.
    Skip,
    /// Change the volume. Volume is represented as a decimal value between 0~1, and the passed value is added to the current volume.
    /// To reduce the value, pass a negative number.
    ChangeVolume(f32),
    /// Sets the volume absolutely.
    SetVolume(f32),

    /// Below should not be used by the user directly.
    AddToQueue {
        music: Vec<Media>,
        pos: QueueLocation,
    },
    PlayURL {
        music: Media,
        url: String,
    },
    Kill,
}

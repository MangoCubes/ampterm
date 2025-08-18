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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToPlayerWorker {
    Stop,
    Pause,
    Continue,
    Skip,
    Previous,
    GoToStart,
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

use serde::{Deserialize, Serialize};
use strum::Display;

use crate::action::getplaylist::Media;

#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum QueueLocation {
    // Music is added before the first element in the queue
    // When this action is invoked, the current song is stopped, and the sent music is added
    Start,
    // Music is added after the first element in the queue
    Next,
    // Music is added at the end of the queue
    Last,
}

#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum PlayerAction {
    Stop,
    Pause,
    Continue,
    Skip,
    AddToQueue { music: Media, pos: QueueLocation },
    PlayURL { music: Media, url: String },
    Kill,
}

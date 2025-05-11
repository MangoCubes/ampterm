use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum PlayerAction {
    Stop,
    Pause,
    TryPlay { url: String },
    Continue,
    Kill,
    Playing,
    Skip,
}

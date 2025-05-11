pub mod setcredential;
use serde::{Deserialize, Serialize};

use setcredential::Credential;
use strum::Display;

use crate::playerworker::player::QueueLocation;

#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum Query {
    // Stop query task
    Kill,
    SetCredential(Credential),
    GetPlaylists,
    GetPlaylist { name: Option<String>, id: String },
    AddToQueueId { pos: QueueLocation, id: String },
    Ping,
}

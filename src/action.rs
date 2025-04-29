pub mod ping;
pub mod playlists;

use ping::PingResponse;
use playlists::PlaylistsResponse;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::queryworker::query::Query;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,

    Query(Query),

    Ping(PingResponse),
    Playlists(PlaylistsResponse),
}

pub mod getplaylist;
pub mod getplaylists;
pub mod ping;

use getplaylist::GetPlaylistResponse;
use getplaylists::GetPlaylistsResponse;
use ping::PingResponse;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::queryworker::query::Query;
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum LocalAction {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Cancel,
}
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
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

    Local(LocalAction),

    SelectPlaylist { key: String },

    Query(Query),

    Ping(PingResponse),
    GetPlaylists(GetPlaylistsResponse),
    GetPlaylist(GetPlaylistResponse),
}

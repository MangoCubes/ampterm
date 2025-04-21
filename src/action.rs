pub mod loginresponse;
pub mod playlistsresponse;

use loginresponse::LoginResponse;
use playlistsresponse::PlaylistsResponse;
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

    Login(LoginResponse),
    Playlists(PlaylistsResponse),
}

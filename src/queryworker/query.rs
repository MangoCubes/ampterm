pub mod playlists;
pub mod setcredential;
use playlists::PlaylistsQuery;
use serde::{Deserialize, Serialize};

use setcredential::Credential;
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Query {
    // Stop query task
    Stop,
    SetCredential(Credential),
    Playlists(PlaylistsQuery),
    Ping,
}

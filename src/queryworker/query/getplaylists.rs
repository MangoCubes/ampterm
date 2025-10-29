use serde::{Deserialize, Serialize};

use crate::osclient::response::getplaylists::SimplePlaylist;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaylistID(String);

impl PlaylistID {
    pub fn new(id: String) -> Self {
        Self(id)
    }
}

impl Clone for PlaylistID {
    fn clone(&self) -> Self {
        PlaylistID(self.0.clone())
    }
}

impl From<PlaylistID> for String {
    fn from(id: PlaylistID) -> Self {
        id.0 // Return the inner String
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum GetPlaylistsResponse {
    Success(Vec<SimplePlaylist>),
    Failure(String),
}

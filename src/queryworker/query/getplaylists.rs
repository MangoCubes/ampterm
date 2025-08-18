use serde::{Deserialize, Serialize};
use strum::Display;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimplePlaylist {
    pub id: PlaylistID,
    pub name: String,
    pub owner: String,
    pub public: bool,
    pub created: String,
    pub changed: String,
    pub song_count: u32,
    pub duration: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum GetPlaylistsResponse {
    Success(Vec<SimplePlaylist>),
    Failure(String),
}

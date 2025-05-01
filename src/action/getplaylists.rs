use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimplePlaylist {
    pub id: String,
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

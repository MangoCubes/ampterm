use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaylistEntry {
    pub id: String,
    pub parent: String,
    pub title: String,
    pub is_dir: bool,
    pub is_video: bool,
    pub entry_type: String,
    pub album_id: String,
    pub album: String,
    pub artist_id: String,
    pub artist: String,
    pub cover_art: String,
    pub duration: u32,
    pub bit_rate: u32,
    pub bit_depth: u32,
    pub sampling_rate: u32,
    pub channel_count: u32,
    pub user_rating: u32,
    pub average_rating: u32,
    pub track: u32,
    pub year: u32,
    pub genre: String,
    pub size: u32,
    pub disc_number: u32,
    pub suffix: String,
    pub content_type: String,
    pub path: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FullPlaylist {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub public: bool,
    pub created: String,
    pub changed: String,
    pub song_count: u32,
    pub duration: u32,
    pub entry: Vec<PlaylistEntry>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GetPlaylistResponse {
    Success(FullPlaylist),
    Failure(String),
}

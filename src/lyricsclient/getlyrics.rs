use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct GetLyricsParams {
    pub track_name: String,
    pub artist_name: Option<String>,
    pub album_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct GetLyricsResponse {
    pub id: u32,
    #[serde(alias = "trackName")]
    pub track_name: String,
    #[serde(alias = "artistName")]
    pub artist_name: String,
    #[serde(alias = "albumName")]
    pub album_name: String,
    pub duration: u32,
    pub instrumental: bool,
    #[serde(alias = "plainLyrics")]
    pub plain_lyrics: String,
    #[serde(alias = "syncedLyrics")]
    pub synced_lyrics: String,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct GetLyricsParams {
    track_name: String,
    artist_name: Option<String>,
    album_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct GetLyricsResponse {
    id: u32,
    #[serde(alias = "trackName")]
    track_name: String,
    #[serde(alias = "artistName")]
    artist_name: String,
    #[serde(alias = "albumName")]
    album_name: String,
    duration: u32,
    instrumental: bool,
    #[serde(alias = "plainLyrics")]
    plain_lyrics: String,
    #[serde(alias = "syncedLyrics")]
    synced_lyrics: String,
}

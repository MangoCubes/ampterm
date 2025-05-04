use serde::{Deserialize, Serialize};

use super::errordata::ErrorData;

#[derive(Debug, Deserialize, Serialize)]
pub struct PlaylistEntry {
    pub id: String,
    pub parent: String,
    pub title: String,
    #[serde(alias = "isDir")]
    pub is_dir: bool,
    #[serde(alias = "isVideo")]
    pub is_video: bool,
    #[serde(alias = "type")]
    pub entry_type: String,
    #[serde(alias = "albumId")]
    pub album_id: String,
    pub album: String,
    #[serde(alias = "artistId")]
    pub artist_id: String,
    pub artist: String,
    #[serde(alias = "coverArt")]
    pub cover_art: String,
    pub duration: u32,
    #[serde(alias = "bitRate")]
    pub bit_rate: u32,
    #[serde(alias = "bitDepth")]
    pub bit_depth: u32,
    #[serde(alias = "samplingRate")]
    pub sampling_rate: u32,
    #[serde(alias = "channelCount")]
    pub channel_count: u32,
    #[serde(alias = "userRating")]
    pub user_rating: u32,
    #[serde(alias = "averageRating")]
    pub average_rating: u32,
    pub track: u32,
    pub year: u32,
    pub genre: String,
    pub size: u32,
    #[serde(alias = "discNumber")]
    pub disc_number: u32,
    pub suffix: String,
    #[serde(alias = "contentType")]
    pub content_type: String,
    pub path: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct FullPlaylist {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub public: bool,
    pub created: String,
    pub changed: String,
    #[serde(alias = "songCount")]
    pub song_count: u32,
    pub duration: u32,
    pub entry: Vec<PlaylistEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status")]
pub enum GetPlaylist {
    #[serde(alias = "ok")]
    Ok { playlist: FullPlaylist },
    #[serde(alias = "failed")]
    Failed { error: ErrorData },
}

use serde::{Deserialize, Serialize};

use super::{getplaylist::Media, oserror::OSError};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct SearchResult3 {
    #[serde(default)]
    pub artist: Vec<ArtistResult>,
    #[serde(default)]
    pub album: Vec<AlbumResult>,
    #[serde(default)]
    pub song: Vec<Media>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ArtistResult {
    pub id: String,
    pub name: String,
    #[serde(alias = "albumCount")]
    pub album_count: Option<u32>,
    #[serde(alias = "coverArt")]
    pub cover_art: Option<String>,
    pub starred: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct AlbumResult {
    pub id: String,
    pub name: String,
    pub artist: Option<String>,
    #[serde(alias = "artistId")]
    pub artist_id: Option<String>,
    #[serde(alias = "coverArt")]
    pub cover_art: Option<String>,
    #[serde(alias = "songCount")]
    pub song_count: Option<u32>,
    pub duration: Option<u32>,
    pub created: Option<String>,
    pub starred: Option<String>,
    pub year: Option<u32>,
    pub genre: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status")]
pub enum Search3 {
    #[serde(alias = "ok")]
    Ok {
        #[serde(alias = "searchResult3")]
        search_result3: SearchResult3,
    },
    #[serde(alias = "failed")]
    Failed { error: OSError },
}

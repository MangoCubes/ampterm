use serde::{Deserialize, Serialize};

use super::errordata::ErrorData;

#[derive(Debug, Deserialize, Serialize)]
pub struct SimplePlaylist {
    id: String,
    name: String,
    owner: String,
    public: bool,
    created: String,
    changed: String,
    #[serde(alias = "songCount")]
    song_count: u32,
    duration: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetPlaylistsWrapper {
    pub playlist: Vec<SimplePlaylist>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status")]
pub enum GetPlaylists {
    #[serde(alias = "ok")]
    Ok { playlists: GetPlaylistsWrapper },
    #[serde(alias = "failed")]
    Failed { error: ErrorData },
}

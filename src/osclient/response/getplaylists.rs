use serde::{Deserialize, Serialize};

use crate::action::getplaylists::PlaylistID;

use super::errordata::ErrorData;

#[derive(Debug, Deserialize, Serialize)]
pub struct SimplePlaylist {
    pub id: PlaylistID,
    pub name: String,
    pub owner: String,
    pub public: bool,
    pub created: String,
    pub changed: String,
    #[serde(alias = "songCount")]
    pub song_count: u32,
    pub duration: u32,
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

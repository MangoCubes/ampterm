use serde::{Deserialize, Serialize};

use crate::queryworker::query::getplaylists::PlaylistID;

use super::oserror::OSError;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct SimplePlaylist {
    pub id: PlaylistID,
    pub name: String,
    pub comment: Option<String>,
    pub owner: Option<String>,
    pub public: Option<bool>,
    #[serde(alias = "songCount")]
    pub song_count: u32,
    pub duration: u32,
    pub created: String,
    pub changed: String,
    #[serde(alias = "coverArt")]
    pub cover_art: Option<String>,
    #[serde(alias = "allowedUsers")]
    pub allowed_users: Option<Vec<String>>,
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
    Failed { error: OSError },
}

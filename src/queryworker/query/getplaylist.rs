use serde::{Deserialize, Serialize};

use crate::{
    osclient::response::getplaylist::FullPlaylist, queryworker::query::getplaylists::PlaylistID,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPlaylistParams {
    // This field is not necessary, but is used to inform user whenever the query fails
    pub name: String,
    pub id: PlaylistID,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GetPlaylistResponse {
    Success(FullPlaylist),
    Failure {
        id: PlaylistID,
        name: String,
        msg: String,
    },
}

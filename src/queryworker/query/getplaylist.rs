use serde::{Deserialize, Serialize};

use crate::{
    osclient::response::getplaylist::FullPlaylist, queryworker::query::getplaylists::PlaylistID,
};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GetPlaylistResponse {
    Success(FullPlaylist),
    Failure {
        id: PlaylistID,
        name: String,
        msg: String,
    },
}

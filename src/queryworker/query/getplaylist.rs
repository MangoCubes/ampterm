use serde::{Deserialize, Serialize};

use crate::osclient::{
    response::{getplaylist::FullPlaylist, getplaylists::SimplePlaylist},
    types::PlaylistID,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetPlaylistParams {
    // This field is not necessary, but is used to inform user whenever the query fails
    pub name: String,
    pub id: PlaylistID,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GetPlaylistResponse {
    Success(FullPlaylist),
    Partial(SimplePlaylist),
    Failure {
        id: PlaylistID,
        name: String,
        msg: String,
    },
}

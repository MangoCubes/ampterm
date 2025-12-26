use serde::{Deserialize, Serialize};

use crate::osclient::response::getplaylists::SimplePlaylist;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum GetPlaylistsResponse {
    Success(Vec<SimplePlaylist>),
    Failure(String),
}

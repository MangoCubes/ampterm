pub mod setcredential;
use serde::{Deserialize, Serialize};

use setcredential::Credential;
use strum::Display;

use crate::action::{getplaylist::Media, getplaylists::PlaylistID};

#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum Query {
    // Stop query task
    Kill,
    SetCredential(Credential),
    GetPlaylists,
    GetPlaylist {
        name: Option<String>,
        id: PlaylistID,
    },
    GetUrlByMedia {
        media: Media,
    },
    Ping,
}

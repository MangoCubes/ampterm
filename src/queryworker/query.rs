pub mod login;
pub mod playlists;
use login::{Credentials, LoginQuery};
use playlists::PlaylistsQuery;
use serde::{Deserialize, Serialize};

use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Query {
    // Stop query task
    Stop,
    SetCredentials(Credentials),
    Login(LoginQuery),
    Playlists(PlaylistsQuery),
}

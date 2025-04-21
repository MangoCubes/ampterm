use serde::{Deserialize, Serialize};

use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum PlaylistsQuery {
    GetPlaylists,
}

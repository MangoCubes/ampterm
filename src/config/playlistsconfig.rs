use std::collections::HashMap;

use serde::Deserialize;

use crate::osclient::types::PlaylistID;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct PlaylistsConfig {
    #[serde(default)]
    pub playlists: HashMap<char, PlaylistID>,
}

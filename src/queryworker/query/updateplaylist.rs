use crate::osclient::types::{MediaID, PlaylistID};

#[derive(Debug, Clone, PartialEq)]
pub struct UpdatePlaylistParams {
    pub playlist_id: PlaylistID,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub public: Option<bool>,
    pub song_id_to_add: Option<Vec<MediaID>>,
    pub song_index_to_remove: Option<Vec<usize>>,
}

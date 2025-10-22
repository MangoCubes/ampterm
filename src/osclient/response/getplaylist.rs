use serde::{Deserialize, Serialize};

use crate::queryworker::query::getplaylists::PlaylistID;

use super::oserror::OSError;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Media {
    pub id: String,
    pub parent: Option<String>,
    #[serde(alias = "isDir")]
    pub is_dir: bool,
    pub title: String,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub track: Option<i32>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    #[serde(alias = "coverArt")]
    pub cover_art: Option<String>,
    pub size: Option<u64>,
    #[serde(alias = "contentType")]
    pub content_type: Option<String>,
    pub suffix: Option<String>,
    #[serde(alias = "transcodedContentType")]
    pub transcoded_content_type: Option<String>,
    #[serde(alias = "transcodedSuffix")]
    pub transcoded_suffix: Option<String>,
    pub duration: Option<i32>,
    #[serde(alias = "bitRate")]
    pub bit_rate: Option<i32>,
    #[serde(alias = "bitDepth")]
    pub bit_depth: Option<i32>,
    #[serde(alias = "samplingRate")]
    pub sampling_rate: Option<i32>,
    #[serde(alias = "channelCount")]
    pub channel_count: Option<i32>,
    pub path: Option<String>,
    #[serde(alias = "isVideo")]
    pub is_video: Option<bool>,
    #[serde(alias = "userRating")]
    pub user_rating: Option<i32>,
    #[serde(alias = "averageRating")]
    pub average_rating: Option<f32>,
    #[serde(alias = "playCount")]
    pub play_count: Option<u64>,
    #[serde(alias = "discNumber")]
    pub disc_number: Option<i32>,
    pub created: Option<String>,
    pub starred: Option<String>,
    #[serde(alias = "albumId")]
    pub album_id: Option<String>,
    #[serde(alias = "artistId")]
    pub artist_id: Option<String>,
    #[serde(alias = "type")]
    pub media_type: Option<String>,
    #[serde(alias = "bookmarkPosition")]
    pub bookmark_position: Option<u64>,
    #[serde(alias = "originalWidth")]
    pub original_width: Option<i32>,
    #[serde(alias = "originalHeight")]
    pub original_height: Option<i32>,
    pub played: Option<String>,
    pub bpm: Option<i32>,
    pub comment: Option<String>,
    #[serde(alias = "sortName")]
    pub sort_name: Option<String>,
    #[serde(alias = "musicBrainzId")]
    pub music_brainz_id: Option<String>,
    // pub genres: Option<Vec<ItemGenre>>,
    // pub artists: Option<Vec<ArtistID3>>,
    #[serde(alias = "displayArtist")]
    pub display_artist: Option<String>,
    // pub album_artists: Option<Vec<ArtistID3>>,
    #[serde(alias = "displayAlbumArtist")]
    pub display_album_artist: Option<String>,
    // pub contributors: Option<Vec<Contributor>>,
    #[serde(alias = "displayComposer")]
    pub display_composer: Option<String>,
    pub moods: Option<Vec<String>>,
    // pub replay_gain: Option<ReplayGain>,
    #[serde(alias = "explicitStatus")]
    pub explicit_status: Option<String>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FullPlaylist {
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
    pub entry: Vec<Media>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status")]
pub enum GetPlaylist {
    #[serde(alias = "ok")]
    Ok { playlist: FullPlaylist },
    #[serde(alias = "failed")]
    Failed { error: OSError },
}

impl Media {
    #[inline(always)]
    pub fn get_fav_marker(&self) -> String {
        if let Some(_) = self.starred {
            "★".to_string()
        } else {
            " ".to_string()
        }
    }
}

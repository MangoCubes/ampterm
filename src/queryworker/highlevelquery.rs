use crate::{
    compid::CompID,
    lyricsclient::getlyrics::GetLyricsParams,
    osclient::{
        response::getplaylist::Media,
        types::{CoverID, MediaID},
    },
    queryworker::query::{
        getplaylist::GetPlaylistParams, setcredential::Credential,
        updateplaylist::UpdatePlaylistParams,
    },
};

/// [`HighLevelQuery`] are sort of a wrapper of normal HTTP queries. These correspond more closely
/// to the actual user actions rather than HTTP request endpoints. As a result, these contain the
/// following information:
///   - The component the request and response should go to
///   - The endpoint that should be invoked
///   - The purpose of the query, and how the response should be handled
///
/// This information is needed to ensure the correct components are updated.
#[derive(Debug, Clone, PartialEq)]
pub enum HighLevelQuery {
    /// Given [`Media`] object, get its URL to play it with [`PlayerWorker`]
    PlayMusicFromURL(Media),
    /// Given a playlist ID, fetch the content to display the musics in a playlist
    SelectPlaylist(GetPlaylistParams),
    /// Fetches the content of a playlist, and add all or part of them to the queue
    AddPlaylistToQueue(GetPlaylistParams),
    /// Fetches the list of playlists for the sake of displaying them
    ListPlaylists,
    /// Fetches the list of playlists for the playlist selection popup
    ListPlaylistsPopup(bool),
    /// Stars/unstars a music
    SetStar {
        media: MediaID,
        star: bool,
    },
    /// Fetch lyrics from lrclib.net
    GetLyrics(GetLyricsParams),
    GetCover(CoverID),
    /// Sets the credential for this client, and sends a ping to ensure it is valid
    Login(Credential),
    UpdatePlaylist(UpdatePlaylistParams),
}

impl HighLevelQuery {
    pub fn get_dest(&self) -> Vec<CompID> {
        match self {
            HighLevelQuery::PlayMusicFromURL(_) => vec![CompID::NowPlaying],
            HighLevelQuery::SelectPlaylist(_) => {
                vec![CompID::PlaylistQueue]
            }
            HighLevelQuery::AddPlaylistToQueue(_) | HighLevelQuery::ListPlaylists => {
                vec![CompID::PlaylistList]
            }
            HighLevelQuery::Login(_) => vec![CompID::Home],
            HighLevelQuery::SetStar { media: _, star: _ } => {
                vec![CompID::PlaylistQueue, CompID::PlayQueue]
            }
            HighLevelQuery::GetLyrics(_) => vec![CompID::Lyrics],
            HighLevelQuery::GetCover(_) => vec![CompID::ImageComp],
            HighLevelQuery::ListPlaylistsPopup(_) => vec![CompID::MainScreen],
            HighLevelQuery::UpdatePlaylist(_) => vec![CompID::MainScreen],
        }
    }
    pub fn show_task(&self) -> bool {
        !matches!(self, HighLevelQuery::PlayMusicFromURL(_))
    }
}

impl ToString for HighLevelQuery {
    fn to_string(&self) -> String {
        match self {
            HighLevelQuery::PlayMusicFromURL(_) => "Loading media from URL",
            HighLevelQuery::SelectPlaylist(_) => "Fetching playlist content",
            HighLevelQuery::AddPlaylistToQueue(_) => "Adding playlist to the queue",
            HighLevelQuery::ListPlaylists => "Fetching all playlists",
            HighLevelQuery::SetStar { media: _, star: _ } => "Toggle favourite status of a music",
            HighLevelQuery::GetLyrics(_) => "Fetching lyrics",
            HighLevelQuery::GetCover(_) => "Fetching cover image",
            HighLevelQuery::Login(_) => "Set login credentials and check validitiy",
            HighLevelQuery::UpdatePlaylist(_) => "Update playlist",
            HighLevelQuery::ListPlaylistsPopup(_) => "Fetching playlists for the popup",
        }
        .to_string()
    }
}

use crate::{
    compid::CompID,
    lyricsclient::getlyrics::GetLyricsParams,
    osclient::response::getplaylist::Media,
    queryworker::query::{
        getcoverart::CoverID,
        getplaylist::{GetPlaylistParams, MediaID},
        setcredential::Credential,
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
    /// Uses ping query to check if the provided credentials is valid or not
    CheckCredentialValidity,
    /// Given a playlist ID, fetch the content to display the musics in a playlist
    SelectPlaylist(GetPlaylistParams),
    /// Fetches the content of a playlist, and add all or part of them to the queue
    AddPlaylistToQueue(GetPlaylistParams),
    /// Fetches the list of playlists for the sake of displaying them
    ListPlaylists,
    /// Not quite a query, but this exists because there already is a way to communicate with
    /// [`QueryWorker`] object and it sort of makes sense to reuse that channel. However, Login
    /// component changes the login credentials, and there may be errors when parsing the
    /// credentials. The error messages are sent as a response to this query.
    SetCredential(Credential),
    /// Stars/unstars a music
    SetStar {
        media: MediaID,
        star: bool,
    },
    /// Fetch lyrics from lrclib.net
    GetLyrics(GetLyricsParams),
    GetCover(CoverID),
    Tick,
}

impl HighLevelQuery {
    pub fn get_dest(&self) -> Vec<CompID> {
        match self {
            HighLevelQuery::PlayMusicFromURL(_) => vec![],
            HighLevelQuery::CheckCredentialValidity => vec![CompID::Home],
            HighLevelQuery::SelectPlaylist(_) => vec![CompID::PlaylistQueue],
            HighLevelQuery::AddPlaylistToQueue(_) | HighLevelQuery::ListPlaylists => {
                vec![CompID::PlaylistList]
            }
            HighLevelQuery::SetCredential(_) => vec![CompID::Login],
            HighLevelQuery::SetStar { media: _, star: _ } => {
                vec![CompID::PlaylistQueue, CompID::PlayQueue]
            }
            HighLevelQuery::GetLyrics(_) => vec![CompID::NowPlaying],
            HighLevelQuery::GetCover(_) => vec![CompID::NowPlaying],
            HighLevelQuery::Tick => vec![],
        }
    }

    pub fn is_internal(&self) -> bool {
        matches!(
            self,
            HighLevelQuery::PlayMusicFromURL(_)
                | HighLevelQuery::SetCredential(_)
                | HighLevelQuery::Tick
        )
    }
}

impl ToString for HighLevelQuery {
    fn to_string(&self) -> String {
        match self {
            HighLevelQuery::PlayMusicFromURL(_) => "Loading media from URL",
            HighLevelQuery::CheckCredentialValidity => "Checking if credentials is valid",
            HighLevelQuery::SelectPlaylist(_) => "Fetching playlist content",
            HighLevelQuery::AddPlaylistToQueue(_) => "Adding playlist to the queue",
            HighLevelQuery::ListPlaylists => "Fetching all playlists",
            HighLevelQuery::SetCredential(_) => "Setting credentials",
            HighLevelQuery::SetStar { media: _, star: _ } => "Toggle favourite status of a music",
            HighLevelQuery::GetLyrics(_) => "Fetching lyrics",
            HighLevelQuery::GetCover(_) => "Fetching cover image",
            HighLevelQuery::Tick => "Tick",
        }
        .to_string()
    }
}

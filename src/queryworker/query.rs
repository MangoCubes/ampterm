pub mod getcoverart;
pub mod getplaylist;
pub mod getplaylists;
pub mod setcredential;

use image::DynamicImage;

use crate::{
    compid::CompID,
    lyricsclient::getlyrics::GetLyricsResponse,
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{getplaylist::GetPlaylistResponse, getplaylists::GetPlaylistsResponse},
        QueryWorker,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct ToQueryWorker {
    /// Specifies the component that should be alerted when the request is both initialised and completed
    /// This ID is limited to a Ratatui component, as requests to [`PlayerWorker`] are implied with
    /// using [`ToPlayerWorker`]
    pub dest: Vec<CompID>,
    /// Uniquely identifies the request and response
    pub ticket: usize,
    /// Query parameters
    pub query: HighLevelQuery,
}

impl ToQueryWorker {
    pub fn new(hlq: HighLevelQuery) -> Self {
        let ticket = QueryWorker::get_ticket();
        Self {
            dest: hlq.get_dest(),
            ticket,
            query: hlq,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResponseType {
    // Responses from the queries
    Star(Result<(), String>),
    Ping(Result<(), String>),
    GetPlaylists(GetPlaylistsResponse),
    GetPlaylist(GetPlaylistResponse),
    SetCredential(Result<(), String>),
    GetLyrics(Result<Option<GetLyricsResponse>, String>),
    GetCover(Result<DynamicImage, String>),
}

#[derive(Debug, Clone)]
pub enum QueryStatus {
    /// Query has been received by the QueryWorker
    Requested(HighLevelQuery),
    /// Query has been aborted
    Aborted(bool),
    /// Query has been finished with external resources
    Finished(ResponseType),
}

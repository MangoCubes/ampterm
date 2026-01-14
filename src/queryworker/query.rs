pub mod getplaylist;
pub mod setcredential;
pub mod updateplaylist;

use image::DynamicImage;

use crate::{
    compid::CompID,
    lyricsclient::getlyrics::GetLyricsResponse,
    osclient::response::getplaylists::SimplePlaylist,
    queryworker::{
        highlevelquery::HighLevelQuery, query::getplaylist::GetPlaylistResponse, QueryWorker,
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
    GetPlaylists(Result<Vec<SimplePlaylist>, String>),
    GetPlaylist(GetPlaylistResponse),
    GetLyrics(Result<Option<GetLyricsResponse>, String>),
    GetCover(Result<DynamicImage, String>),
    Login(Result<(), String>),
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

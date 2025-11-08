pub mod getplaylist;
pub mod getplaylists;
pub mod setcredential;
use serde::{Deserialize, Serialize};

use crate::{
    compid::CompID,
    lyricsclient::getlyrics::GetLyricsResponse,
    queryworker::{
        highlevelquery::HighLevelQuery,
        query::{getplaylist::GetPlaylistResponse, getplaylists::GetPlaylistsResponse},
        QueryWorker,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResponseType {
    // Responses from the queries
    Star(Result<(), String>),
    Ping(Result<(), String>),
    GetPlaylists(GetPlaylistsResponse),
    GetPlaylist(GetPlaylistResponse),
    SetCredential(Result<(), String>),
    GetLyrics(Result<Option<GetLyricsResponse>, String>),
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FromQueryWorker {
    /// When a query is completed, the value in [`dest`] specifies which component should be
    /// notified. This value should be the same as the corresponding [`ToQueryWorker`] request.
    pub dest: Vec<CompID>,
    /// Uniquely identifies the request and response
    pub ticket: usize,
    /// Actual response body
    pub res: ResponseType,
}

impl FromQueryWorker {
    pub fn new(dest: Vec<CompID>, ticket: usize, res: ResponseType) -> Self {
        Self { dest, ticket, res }
    }
}

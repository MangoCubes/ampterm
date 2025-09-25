pub mod getplaylist;
pub mod getplaylists;
pub mod ping;
pub mod setcredential;
use serde::{Deserialize, Serialize};

use setcredential::Credential;

use crate::{
    osclient::response::getplaylist::Media,
    queryworker::{
        query::{
            getplaylist::GetPlaylistResponse,
            getplaylists::{GetPlaylistsResponse, PlaylistID},
            ping::PingResponse,
        },
        QueryWorker,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    SetCredential(Credential),
    GetPlaylists,
    GetPlaylist { name: String, id: PlaylistID },
    GetUrlByMedia { media: Media },
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToQueryWorker {
    pub dest: u32,
    pub ticket: usize,
    pub query: QueryType,
}

impl ToQueryWorker {
    pub fn new(dest: u32, query: QueryType) -> Self {
        let ticket = QueryWorker::get_ticket();
        Self {
            dest,
            ticket,
            query,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseType {
    // Responses from the queries
    Ping(PingResponse),
    GetPlaylists(GetPlaylistsResponse),
    GetPlaylist(GetPlaylistResponse),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FromQueryWorker {
    pub dest: u32,
    pub ticket: usize,
    pub res: ResponseType,
}

impl FromQueryWorker {
    pub fn new(dest: u32, ticket: usize, query: ResponseType) -> Self {
        Self {
            dest,
            ticket,
            res: query,
        }
    }
}

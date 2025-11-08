use reqwest::{Error, StatusCode};

use crate::lyricsclient::getlyrics::{GetLyricsParams, GetLyricsResponse};

pub mod getlyrics;
pub mod lrclib;
pub enum FailReason {
    URLParsing,
    ErrStatus(StatusCode),
    Text,
    Decoding,
    Querying(Error),
}

pub trait LyricsClient {
    async fn search(
        &self,
        params: GetLyricsParams,
    ) -> Result<Option<GetLyricsResponse>, FailReason>;
}

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
impl FailReason {
    pub fn to_string(&self) -> String {
        match self {
            FailReason::URLParsing => "Failed to parse URL; Please check your config.".to_string(),
            FailReason::ErrStatus(status_code) => {
                format!("Server responded with error code {}", status_code)
            }
            FailReason::Text => "Failed to extract response body.".to_string(),
            FailReason::Decoding => "Failed to decode response body. It is possible that this happened because the provider changed their API interface.".to_string(),
            FailReason::Querying(error) => format!("Failed to send query: {}", error),
        }
    }
}

pub trait LyricsClient {
    async fn search(
        &self,
        params: GetLyricsParams,
    ) -> Result<Option<GetLyricsResponse>, FailReason>;
}

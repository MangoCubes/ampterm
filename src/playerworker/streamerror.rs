use std::{error::Error, fmt::Display};
use stream_download::{
    http::{HttpStream, HttpStreamError},
    StreamInitializationError,
};
use tokio::task::JoinError;

#[derive(Debug)]
enum ErrType {
    Parse(String),
    Stream(HttpStreamError<reqwest::Client>),
    StreamInit(StreamInitializationError<HttpStream<reqwest::Client>>),
    Join(JoinError), // Rodio(rodio::StreamError),
                     // Decode(rodio::decoder::DecoderError),
                     // Play(rodio::PlayError),
}
impl Display for ErrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrType::Stream(e) => write!(f, "Stream Error: {}", e),
            ErrType::StreamInit(e) => write!(f, "Stream Initialisation Error: {}", e),
            ErrType::Parse(url) => write!(f, "URL Parsing Error: {}", url),
            ErrType::Join(e) => write!(f, "Join Error: {}", e),
            // ErrType::Rodio(e) => write!(f, "Output stream error: {}", e),
            // ErrType::Decode(e) => write!(f, "Decode error: {}", e),
            // ErrType::Play(e) => write!(f, "Player error: {}", e),
        }
    }
}

// This error is given when the error occurs during the querying
// In other words, if this error is shown to the user, it means that the media server has not been
// involved.
#[derive(Debug)]
pub struct StreamError {
    reason: ErrType,
}

impl Error for StreamError {}
impl Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.reason.fmt(f)
    }
}
impl StreamError {
    pub fn stream(e: HttpStreamError<reqwest::Client>) -> StreamError {
        Self {
            reason: ErrType::Stream(e),
        }
    }

    pub fn stream_init(e: StreamInitializationError<HttpStream<reqwest::Client>>) -> StreamError {
        Self {
            reason: ErrType::StreamInit(e),
        }
    }

    pub fn parse(e: String) -> StreamError {
        Self {
            reason: ErrType::Parse(e),
        }
    }

    pub fn join(e: JoinError) -> StreamError {
        Self {
            reason: ErrType::Join(e),
        }
    }
    // pub fn rodio(e: rodio::StreamError) -> StreamError {
    //     Self {
    //         reason: ErrType::Rodio(e),
    //     }
    // }
    //
    // pub fn play(e: rodio::PlayError) -> StreamError {
    //     Self {
    //         reason: ErrType::Play(e),
    //     }
    // }
    //
    // pub fn decode(e: rodio::decoder::DecoderError) -> StreamError {
    //     Self {
    //         reason: ErrType::Decode(e),
    //     }
    // }
}

use std::{error::Error, fmt::Display};

use reqwest::StatusCode;

#[derive(Debug)]
enum ErrType {
    Request(reqwest::Error),
    Decode(serde_json::Error),
    Response(StatusCode),
}
impl Display for ErrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrType::Request(e) => write!(f, "Request error: {}", e),
            ErrType::Decode(e) => write!(f, "Decode error: {}", e),
            ErrType::Response(e) => write!(f, "Server responded with status code: {}", e),
        }
    }
}

// This error is given when the error occurs during the querying
// In other words, the cause that triggered this error does not involve the media server
#[derive(Debug)]
pub struct ExternalError {
    reason: ErrType,
}

impl Error for ExternalError {}
impl Display for ExternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.reason.fmt(f)
    }
}
impl ExternalError {
    // Error has happened before sending the request to the server
    pub fn req(e: reqwest::Error) -> ExternalError {
        Self {
            reason: ErrType::Request(e),
        }
    }
    // Error has occurred after sending the request to the server (4XX, 5XX, etc)
    pub fn res(e: StatusCode) -> ExternalError {
        Self {
            reason: ErrType::Response(e),
        }
    }
    pub fn decode(e: serde_json::Error) -> ExternalError {
        Self {
            reason: ErrType::Decode(e),
        }
    }
}

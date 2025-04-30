use std::{error::Error, fmt::Display};

#[derive(Debug)]
enum ErrType {
    Request(reqwest::Error),
}
impl Display for ErrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrType::Request(e) => write!(f, "Request Error: {}", e),
        }
    }
}

// This error is given when the error occurs during the querying
// In other words, if this error is shown to the user, it means that the media server has not been
// involved.
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
    pub fn new(e: reqwest::Error) -> ExternalError {
        Self {
            reason: ErrType::Request(e),
        }
    }
}

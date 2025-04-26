use std::{error::Error, fmt::Display};

#[derive(Debug)]
enum ErrType {
    Request(reqwest::Error),
}

// This error is given when the error occurs during the querying
// In other words, if this error is shown to the user, it means that the media server has not been
// involved.
#[derive(Debug)]
pub struct GeneralError {
    errtype: ErrType,
}

impl Error for GeneralError {}
impl Display for GeneralError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
impl GeneralError {
    pub fn new(e: reqwest::Error) -> GeneralError {
        Self {
            errtype: ErrType::Request(e),
        }
    }
}

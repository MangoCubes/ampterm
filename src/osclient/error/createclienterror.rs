use std::{error::Error, fmt::Display};

use crate::osclient::response::errordata::ErrorData;

use super::externalerror::ExternalError;

#[derive(Debug)]
pub enum CreateClientFailureReason {
    // Login failed response came from the server (invalid credentials)
    Internal(ErrorData),
    // Error arrived from external library (JSON decoding, too many redirects, etc)
    External(ExternalError),
}

#[derive(Debug)]
pub struct CreateClientError {
    reason: CreateClientFailureReason,
}

impl CreateClientError {
    pub fn external(reason: ExternalError) -> Self {
        Self {
            reason: CreateClientFailureReason::External(reason),
        }
    }

    pub fn internal(reason: ErrorData) -> Self {
        Self {
            reason: CreateClientFailureReason::Internal(reason),
        }
    }
}

impl Error for CreateClientError {}

impl Display for CreateClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

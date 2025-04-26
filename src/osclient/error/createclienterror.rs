use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum CreateClientFailureReason {
    // Login fails due to invalid host
    InvalidURL,
    // Login fails due to invalid credentials (Wrong username or password)
    InvalidCredentials,
    // Failed to ping the server
    FailedPing,
}
#[derive(Debug)]
pub struct CreateClientError {
    reason: CreateClientFailureReason,
}

impl Error for CreateClientError {}

impl Display for CreateClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

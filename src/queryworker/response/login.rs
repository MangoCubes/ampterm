use serde::{Deserialize, Serialize};

use strum::Display;
#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub(crate) enum LoginResponse {
    // Login was successful
    Success,
    // Login fails due to invalid host
    InvalidURL,
    // Login fails due to invalid credentials (Wrong username or password)
    InvalidCredentials,
    // Failed to ping the server
    FailedPing,
    // Login fails due to other reasons not listed here
    Other(String),
}

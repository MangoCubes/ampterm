use serde::{Deserialize, Serialize};

use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub(crate) enum LoginAction {
    Success(String, String, String),
    InvalidURL,
    InvalidCredentials,
    Other(String),
}

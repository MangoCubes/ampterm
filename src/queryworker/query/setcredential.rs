use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Credential {
    // Use your password to log in
    Password {
        url: String,
        secure: bool,
        username: String,
        password: String,
        legacy: bool,
    },
    // Use API key to log in
    APIKey {
        url: String,
        secure: bool,
        username: String,
        apikey: String,
    },
}

impl Credential {
    pub fn get_username(&self) -> String {
        match self {
            Credential::Password {
                url: _,
                secure: _,
                username,
                password: _,
                legacy: _,
            } => username.clone(),
            Credential::APIKey {
                url: _,
                secure: _,
                username,
                apikey: _,
            } => username.clone(),
        }
    }
    pub fn get_url(&self) -> String {
        match self {
            Credential::Password {
                url,
                secure: _,
                username: _,
                password: _,
                legacy: _,
            } => url.clone(),
            Credential::APIKey {
                url,
                secure: _,
                username: _,
                apikey: _,
            } => url.clone(),
        }
    }
}

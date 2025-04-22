use error::createclienterror::CreateClientError;
use serde::{Deserialize, Serialize};
use strum::Display;

mod error;
#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Credential {
    // Use your password to log in
    Password {
        url: String,
        username: String,
        password: String,
        legacy: bool,
    },
    // Use API key to log in
    APIKey {
        url: String,
        username: String,
        apikey: String,
    },
}

pub struct Client {
    auth: Credential,
}

impl Client {
    // Use token to create a client
    // A ping request is sent with the credentials to verify it
    // Will fail if the credentials is wrong
    pub fn password(
        url: String,
        username: String,
        password: String,
        legacy: bool,
    ) -> Result<Client, CreateClientError> {
        let client = Client::use_password(url, username, password, legacy);
        Ok(client)
    }
    // Use token to create a client
    // A ping request is sent with the credentials to verify it
    // Will fail if the credentials is wrong
    pub fn token(url: String, username: String, apikey: String) -> Result<Self, CreateClientError> {
        let client = Client::use_token(url, username, apikey);
        Ok(client)
    }
    // Use token to create a client
    // This bypasses the credentials check, and will always return Client
    pub fn use_token(url: String, username: String, apikey: String) -> Self {
        Self {
            auth: Credential::APIKey {
                url,
                username,
                apikey,
            },
        }
    }
    // Use password to create a client
    // This bypasses the credentials check, and will always return Client
    pub fn use_password(url: String, username: String, password: String, legacy: bool) -> Self {
        Self {
            auth: Credential::Password {
                url,
                username,
                password,
                legacy,
            },
        }
    }
}

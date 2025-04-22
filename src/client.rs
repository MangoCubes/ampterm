use error::createclienterror::CreateClientError;

mod error;
enum Credential {
    // Use your password to log in
    // If the second parameter is true, then legacy authentication (send password as-is instead of
    // hash) is used
    Password(String, bool),
    APIKey(String),
}

pub struct Client {
    url: String,
    username: String,
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
            url,
            username,
            auth: Credential::APIKey(apikey),
        }
    }
    // Use password to create a client
    // This bypasses the credentials check, and will always return Client
    pub fn use_password(url: String, username: String, password: String, legacy: bool) -> Self {
        Self {
            url,
            username,
            auth: Credential::Password(password, legacy),
        }
    }
}

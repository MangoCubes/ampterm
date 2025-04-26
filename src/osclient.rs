use error::createclienterror::CreateClientError;
use reqwest::{Client, Url};

pub mod error;
#[derive(Debug)]
pub enum Credential {
    // Use your password to log in
    Password {
        url: Url,
        secure: bool,
        username: String,
        password: String,
        legacy: bool,
    },
    // Use API key to log in
    APIKey {
        url: Url,
        secure: bool,
        username: String,
        apikey: String,
    },
}

pub struct OSClient {
    auth: Credential,
    client: Client,
}

impl OSClient {
    async fn ping() {}
    // Use token to create a client
    // A ping request is sent with the credentials to verify it
    // Will fail if the credentials is wrong
    pub async fn password(
        url: String,
        username: String,
        password: String,
        legacy: bool,
        secure: bool,
    ) -> Result<OSClient, CreateClientError> {
        let client = OSClient::credentials(Credential::Password {
            url: Self::parse_url(&url, secure),
            secure,
            username,
            password,
            legacy,
        })
        .await?;
        Ok(client)
    }
    // Use token to create a client
    // A ping request is sent with the credentials to verify it
    // Will fail if the credentials is wrong
    pub async fn token(
        url: String,
        username: String,
        apikey: String,
        secure: bool,
    ) -> Result<Self, CreateClientError> {
        let client = OSClient::credentials(Credential::APIKey {
            url: Self::parse_url(&url, secure),
            secure,
            username,
            apikey,
        })
        .await?;
        Ok(client)
    }
    pub async fn credentials(auth: Credential) -> Result<Self, CreateClientError> {
        let client = OSClient::use_credentials(auth);
        Ok(client)
    }
    pub fn use_credentials(auth: Credential) -> Self {
        Self {
            auth,
            client: Client::builder()
                .build()
                .expect("Failed to create reqwest client."),
        }
    }
    fn parse_url(url: &str, secure: bool) -> Url {
        let mut url = Url::parse(url).expect("Failed to parse the URL.");
        url.set_scheme(if secure { "https" } else { "http" });
        url
    }
}

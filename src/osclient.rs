use error::{createclienterror::CreateClientError, externalerror::ExternalError};
use reqwest::Method;
use reqwest::{Client, Url};
use response::empty::Empty;
use response::wrapper::Wrapper;
use serde::de::DeserializeOwned;

pub mod error;
pub mod response;

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
    async fn ping(&self) -> Result<Empty, ExternalError> {
        self.query_auth::<Empty>(Method::GET, "ping").await
    }
    // Make a request to an arbitrary endpoint and get its result
    async fn query_auth<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
    ) -> Result<T, ExternalError> {
        fn get_path(url: &Url, name: &str, secure: bool) -> Url {
            let path = &format!("api/{}", name);
            let mut ret = url.clone();
            ret.set_path(path);
            ret.set_scheme(if secure { "https" } else { "http" });
            ret
        }
        let r = match &self.auth {
            Credential::Password {
                url,
                secure,
                username,
                password,
                legacy,
            } => {
                let params: Vec<(&str, &str)> = if *legacy {
                    vec![
                        ("u", &username),
                        ("p", &password),
                        ("v", "1.16.1"),
                        ("c", "ampterm-client"),
                        ("f", "json"),
                    ]
                } else {
                    vec![
                        ("u", &username),
                        ("t", todo!()),
                        ("s", todo!()),
                        ("v", "1.16.1"),
                        ("c", "ampterm-client"),
                        ("f", "json"),
                    ]
                };
                self.client
                    .request(method, get_path(&url, path, *secure))
                    .query(&params)
                    .send()
                    .await
            }
            Credential::APIKey {
                url,
                secure,
                username,
                apikey,
            } => {
                todo!()
            }
        };
        let handler = |e: reqwest::Error| ExternalError::new(e);
        let data = r
            .map_err(handler)?
            .json::<Wrapper<T>>()
            .await
            .map_err(handler)?;
        Ok(data.subsonic_response)
    }
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
            url: Url::parse(&url).expect("Failed to parse the URL."),
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
            url: Url::parse(&url).expect("Failed to parse the URL."),
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
}

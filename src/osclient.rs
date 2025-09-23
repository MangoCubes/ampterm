use crate::trace_dbg;
use error::{createclienterror::CreateClientError, externalerror::ExternalError};
use reqwest::{Client, Url};
use reqwest::{Method, Response};
use response::empty::Empty;
use response::getplaylist::GetPlaylist;
use response::getplaylists::GetPlaylists;
use response::wrapper::Wrapper;
use serde::de::DeserializeOwned;
use serde_json::from_str;
use std::fmt::Debug;

mod error;
pub mod response;

#[derive(Debug)]
enum Credential {
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

// For every API request, the return value is as follows:
// Result<Result<Success, Failure>, ExternalError>
// If ExternalError is received, it means that the library this library is depending on has failed.
// Examples include invalid hostname, infinite redirect, etc
// If Result<Success, Failure> is received, the server has responded.
// If Success, the data is in the response if the API returns them
// If Failure, the server has decided not to give the client the information requested for various
// reasons, which can be found within the response.
//
// Currently, there are two exception to this rule,
// 1. Client creation: Returns Result<Client, CreateClientError>
// This one always return client if
//   A. The library has worked successfully
//   B. The ping is successful
// If any of these condition is not met, then CreateClientError is returned
// In other words, the Failure is bundled with ExternalError instead of with Success
// This is because I would expect client creating function to return the client, and not a Ping
// response.
//
// 2. Any queries that do not require queries
// stream_link is an example of this. It returns a link from which a client can stream data

impl OSClient {
    pub fn stream_link(&self, id: String) -> Url {
        self.get_path("stream", Some(vec![("id", &id)]))
    }
    pub async fn get_playlist(&self, id: String) -> Result<GetPlaylist, ExternalError> {
        self.query_auth::<GetPlaylist>(Method::GET, "getPlaylist", Some(vec![("id", &id)]))
            .await
    }
    pub async fn get_playlists(&self) -> Result<GetPlaylists, ExternalError> {
        self.query_auth::<GetPlaylists>(Method::GET, "getPlaylists", None)
            .await
    }
    pub async fn ping(&self) -> Result<Empty, ExternalError> {
        self.query_auth::<Empty>(Method::GET, "ping", None).await
    }
    fn get_path(&self, path: &str, query: Option<Vec<(&str, &str)>>) -> Url {
        let (mut params, mut url): (Vec<(&str, &str)>, Url) = match &self.auth {
            Credential::Password {
                url,
                secure,
                username,
                password,
                legacy,
            } => {
                let mut ret = url.clone();
                let _ = ret.set_scheme(if *secure { "https" } else { "http" });
                (
                    if *legacy {
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
                    },
                    ret,
                )
            }
            Credential::APIKey {
                url,
                username,
                apikey,
                secure,
            } => {
                let mut ret = url.clone();
                let _ = ret.set_scheme(if *secure { "https" } else { "http" });
                (
                    vec![
                        ("u", &username),
                        ("apiKey", &apikey),
                        ("v", "1.16.1"),
                        ("c", "ampterm-client"),
                        ("f", "json"),
                    ],
                    ret,
                )
            }
        };
        if let Some(a) = query {
            params.extend(a);
        };
        let path = &format!("rest/{}", path);
        url.set_path(path);
        params.into_iter().for_each(|(k, v)| {
            url.query_pairs_mut().append_pair(k, v);
        });
        url
    }
    async fn query(
        &self,
        method: Method,
        path: &str,
        query: Option<Vec<(&str, &str)>>,
    ) -> Result<Response, reqwest::Error> {
        self.client
            .request(method, self.get_path(path, query))
            .send()
            .await
    }
    // Make a request to an arbitrary endpoint and get its result
    async fn query_auth<T: DeserializeOwned + Debug>(
        &self,
        method: Method,
        path: &str,
        query: Option<Vec<(&str, &str)>>,
    ) -> Result<T, ExternalError> {
        let r = self.query(method, path, query).await;
        let handler = |e: reqwest::Error| ExternalError::req(e);
        let response = r.map_err(handler)?;
        match response.error_for_status() {
            Ok(r) => {
                let body = r.text().await.map_err(handler)?;
                let data = from_str::<Wrapper<T>>(&body).map_err(|e| ExternalError::decode(e))?;
                Ok(trace_dbg!(data.subsonic_response))
            }
            Err(status) => {
                if let Some(code) = status.status() {
                    Err(trace_dbg!(ExternalError::res(code)))
                } else {
                    panic!("Received error status code but found no error status code??")
                }
            }
        }
    }
    // Use password to create a client without verifying if the credentials are valid
    pub fn use_password(
        url: String,
        username: String,
        password: String,
        legacy: bool,
        secure: bool,
    ) -> Self {
        OSClient::use_credentials(Credential::Password {
            url: Url::parse(&url).expect("Failed to parse the URL."),
            secure,
            username,
            password,
            legacy,
        })
    }
    // Use API key to create a client without verifying if the credentials are valid
    pub fn use_apikey(url: String, username: String, apikey: String, secure: bool) -> Self {
        OSClient::use_credentials(Credential::APIKey {
            url: Url::parse(&url).expect("Failed to parse the URL."),
            secure,
            username,
            apikey,
        })
    }
    // Use password to create a client
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
    // Use API key to create a client
    // A ping request is sent with the credentials to verify it
    // Will fail if the credentials is wrong
    pub async fn apikey(
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
    // Create a client using the given credentials, then ping the server to verify the result
    pub async fn credentials(auth: Credential) -> Result<Self, CreateClientError> {
        let client = OSClient::use_credentials(auth);
        let ping_result = client
            .ping()
            .await
            .map_err(|e| CreateClientError::external(e))?;
        match ping_result {
            Empty::Ok => Ok(client),
            Empty::Failed { error } => Err(CreateClientError::internal(error)),
        }
    }
    // Create a client using the given credentials
    // Unlike OSClient::credentials, this one does not verify if the credentials are valid
    pub fn use_credentials(auth: Credential) -> Self {
        Self {
            auth,
            client: Client::builder()
                .build()
                .expect("Failed to create reqwest client."),
        }
    }
}

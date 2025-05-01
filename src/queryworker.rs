pub mod query;

use std::sync::Arc;

use color_eyre::{eyre, Result};
use query::setcredential::Credential;
use query::Query;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::action::getplaylists::{GetPlaylistsResponse, SimplePlaylist};
use crate::action::ping::PingResponse;
use crate::action::Action;
use crate::config::Config;
use crate::osclient::response::empty::Empty;
use crate::osclient::response::getplaylists::GetPlaylists;
use crate::osclient::OSClient;
use crate::trace_dbg;

pub struct QueryWorker {
    client: Option<Arc<OSClient>>,
    req_tx: UnboundedSender<Query>,
    req_rx: UnboundedReceiver<Query>,
    action_tx: UnboundedSender<Action>,
    should_quit: bool,
    config: Config,
}

impl QueryWorker {
    pub async fn get_playlists(c: Arc<OSClient>) -> GetPlaylistsResponse {
        match c.get_playlists().await {
            Ok(r) => match r {
                GetPlaylists::Ok { playlists } => GetPlaylistsResponse::Success(
                    playlists
                        .playlist
                        .into_iter()
                        .map(|p| SimplePlaylist {
                            id: p.id,
                            name: p.name,
                            owner: p.owner,
                            public: p.public,
                            created: p.created,
                            changed: p.changed,
                            song_count: p.song_count,
                            duration: p.duration,
                        })
                        .collect(),
                ),
                GetPlaylists::Failed { error } => GetPlaylistsResponse::Failure(error.to_string()),
            },
            Err(e) => GetPlaylistsResponse::Failure(e.to_string()),
        }
    }
    pub async fn run(&mut self) -> Result<()> {
        trace_dbg!("Starting QueryWorker...");
        loop {
            let Some(event) = self.req_rx.recv().await else {
                break;
            };
            match event {
                Query::Stop => self.should_quit = true,
                Query::SetCredential(creds) => {
                    self.client = Some(Arc::from(match creds {
                        Credential::Password {
                            url,
                            secure,
                            username,
                            password,
                            legacy,
                        } => OSClient::use_password(url, username, password, legacy, secure),
                        Credential::APIKey {
                            url,
                            secure,
                            username,
                            apikey,
                        } => OSClient::use_apikey(url, username, apikey, secure),
                    }));
                }
                Query::GetPlaylists => {
                    let tx = self.action_tx.clone();
                    match &self.client {
                        Some(c) => {
                            let cc = c.clone();
                            tokio::spawn(async move {
                                let res = QueryWorker::get_playlists(cc).await;
                                tx.send(Action::GetPlaylists(res))
                            });
                        }
                        None => tracing::error!(
                            "Invalid state: Tried querying, but client does not exist!"
                        ),
                    };
                }
                Query::Ping => {
                    let tx = self.action_tx.clone();
                    match &self.client {
                        Some(client) => {
                            let c = client.clone();
                            tokio::spawn(async move {
                                let ping = c.ping().await;
                                match ping {
                                    Ok(c) => match c {
                                        Empty::Ok => tx.send(Action::Ping(PingResponse::Success)),
                                        Empty::Failed { error } => tx.send(Action::Ping(
                                            PingResponse::Failure(error.to_string()),
                                        )),
                                    },
                                    Err(e) => tx.send(Action::Ping(PingResponse::Failure(
                                        format!("{}", e),
                                    ))),
                                }
                            });
                        }
                        None => tracing::error!(
                            "Invalid state: Tried querying, but client does not exist!"
                        ),
                    }
                }
            };
            if self.should_quit {
                break;
            }
        }
        Ok(())
    }
}

impl QueryWorker {
    pub fn new(sender: UnboundedSender<Action>, config: Config) -> Self {
        let (req_tx, req_rx) = mpsc::unbounded_channel();
        Self {
            client: None,
            req_tx,
            req_rx,
            action_tx: sender,
            should_quit: false,
            config,
        }
    }
    pub fn get_tx(&self) -> UnboundedSender<Query> {
        self.req_tx.clone()
    }
}

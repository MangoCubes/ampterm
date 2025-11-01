pub mod highlevelquery;
pub mod query;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::action::Action;
use crate::config::Config;
use crate::osclient::response::empty::Empty;
use crate::osclient::response::getplaylist::{GetPlaylist, IndeterminedPlaylist};
use crate::osclient::response::getplaylists::GetPlaylists;
use crate::osclient::OSClient;
use crate::playerworker::player::ToPlayerWorker;
use crate::queryworker::highlevelquery::HighLevelQuery;
use crate::queryworker::query::getplaylist::GetPlaylistResponse;
use crate::queryworker::query::getplaylists::GetPlaylistsResponse;
use crate::queryworker::query::ping::PingResponse;
use crate::queryworker::query::setcredential::Credential;
use crate::queryworker::query::{FromQueryWorker, ResponseType};
use crate::trace_dbg;
use color_eyre::Result;
use query::ToQueryWorker;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub struct QueryWorker {
    client: Option<Arc<OSClient>>,
    req_tx: UnboundedSender<ToQueryWorker>,
    req_rx: UnboundedReceiver<ToQueryWorker>,
    action_tx: UnboundedSender<Action>,
    should_quit: bool,
    config: Config,
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

impl QueryWorker {
    /// Returns a unique ticket number
    /// This value must be included in every request sent to this worker
    pub fn get_ticket() -> usize {
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn get_playlists(c: Arc<OSClient>) -> GetPlaylistsResponse {
        match c.get_playlists().await {
            Ok(r) => match r {
                GetPlaylists::Ok { playlists } => GetPlaylistsResponse::Success(playlists.playlist),
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
            match event.query {
                HighLevelQuery::SetCredential(creds) => {
                    let client = match creds {
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
                    };

                    match client {
                        Ok(client) => {
                            self.client = Some(Arc::from(client));
                            let _ =
                                self.action_tx
                                    .send(Action::FromQueryWorker(FromQueryWorker::new(
                                        event.dest,
                                        event.ticket,
                                        ResponseType::SetCredential(Ok(())),
                                    )));
                        }
                        Err(err) => {
                            let _ =
                                self.action_tx
                                    .send(Action::FromQueryWorker(FromQueryWorker::new(
                                        event.dest,
                                        event.ticket,
                                        ResponseType::SetCredential(Err(err.to_string())),
                                    )));
                            // return Err(eyre!("Failed to log in: {}", err.to_string()));
                        }
                    };
                }
                HighLevelQuery::ListPlaylists => {
                    let tx = self.action_tx.clone();
                    match &self.client {
                        Some(c) => {
                            let cc = c.clone();
                            tokio::spawn(async move {
                                let res = QueryWorker::get_playlists(cc).await;
                                tx.send(Action::FromQueryWorker(FromQueryWorker::new(
                                    event.dest,
                                    event.ticket,
                                    ResponseType::GetPlaylists(res),
                                )))
                            });
                        }
                        None => tracing::error!(
                            "Invalid state: Tried querying, but client does not exist!"
                        ),
                    };
                }
                HighLevelQuery::CheckCredentialValidity => {
                    let tx = self.action_tx.clone();
                    match &self.client {
                        Some(client) => {
                            let c = client.clone();
                            tokio::spawn(async move {
                                let ping = c.ping().await;
                                match ping {
                                    Ok(c) => match c {
                                        Empty::Ok => {
                                            tx.send(Action::FromQueryWorker(FromQueryWorker::new(
                                                event.dest,
                                                event.ticket,
                                                ResponseType::Ping(PingResponse::Success),
                                            )))
                                        }
                                        Empty::Failed { error } => {
                                            tx.send(Action::FromQueryWorker(FromQueryWorker::new(
                                                event.dest,
                                                event.ticket,
                                                ResponseType::Ping(PingResponse::Failure(
                                                    error.to_string(),
                                                )),
                                            )))
                                        }
                                    },
                                    Err(e) => {
                                        tx.send(Action::FromQueryWorker(FromQueryWorker::new(
                                            event.dest,
                                            event.ticket,
                                            ResponseType::Ping(PingResponse::Failure(format!(
                                                "{}",
                                                e
                                            ))),
                                        )))
                                    }
                                }
                            });
                        }
                        None => tracing::error!(
                            "Invalid state: Tried querying, but client does not exist!"
                        ),
                    }
                }
                HighLevelQuery::SelectPlaylist(params)
                | HighLevelQuery::AddPlaylistToQueue(params) => {
                    match &self.client {
                        Some(c) => {
                            let tx = self.action_tx.clone();
                            let client = c.clone();
                            tokio::spawn(async move {
                                let res =
                                    client.get_playlist(String::from(params.id.clone())).await;
                                match res {
                                    Ok(c) => match c {
                                        GetPlaylist::Ok { playlist } => match playlist {
                                            IndeterminedPlaylist::FullPlaylist(full_playlist) => tx
                                                .send(Action::FromQueryWorker(
                                                    FromQueryWorker::new(
                                                        event.dest,
                                                        event.ticket,
                                                        ResponseType::GetPlaylist(
                                                            GetPlaylistResponse::Success(
                                                                full_playlist,
                                                            ),
                                                        ),
                                                    ),
                                                )),
                                            IndeterminedPlaylist::SimplePlaylist(
                                                simple_playlist,
                                            ) => {
                                                let res = match simple_playlist.get(0) {
                                                    Some(playlist) => {
                                                        let pl = playlist.to_owned();
                                                        GetPlaylistResponse::Partial(pl)
                                                    }

                                                    None => GetPlaylistResponse::Failure {
                                                        id: params.id,
                                                        name: params.name,
                                                        msg: "Playlist not found!".to_string(),
                                                    },
                                                };
                                                tx.send(Action::FromQueryWorker(
                                                    FromQueryWorker::new(
                                                        event.dest,
                                                        event.ticket,
                                                        ResponseType::GetPlaylist(res),
                                                    ),
                                                ))
                                            }
                                        },

                                        GetPlaylist::Failed { error } => {
                                            tx.send(Action::FromQueryWorker(FromQueryWorker::new(
                                                event.dest,
                                                event.ticket,
                                                ResponseType::GetPlaylist(
                                                    GetPlaylistResponse::Failure {
                                                        id: params.id,
                                                        name: params.name,
                                                        msg: error.to_string(),
                                                    },
                                                ),
                                            )))
                                        }
                                    },
                                    Err(e) => {
                                        tx.send(Action::FromQueryWorker(FromQueryWorker::new(
                                            event.dest,
                                            event.ticket,
                                            ResponseType::GetPlaylist(
                                                GetPlaylistResponse::Failure {
                                                    id: params.id,
                                                    name: params.name,
                                                    msg: e.to_string(),
                                                },
                                            ),
                                        )))
                                    }
                                }
                            });
                        }
                        None => tracing::error!(
                            "Invalid state: Tried querying, but client does not exist!"
                        ),
                    };
                }
                HighLevelQuery::PlayMusicFromURL(media) => {
                    match &self.client {
                        Some(c) => {
                            let id = media.id.clone();
                            let url = c.stream_link(id).to_string();
                            let _ = self.action_tx.send(Action::ToPlayerWorker(
                                ToPlayerWorker::PlayURL { music: media, url },
                            ));
                        }
                        None => tracing::error!(
                            "Invalid state: Tried querying, but client does not exist!"
                        ),
                    };
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
    pub fn get_tx(&self) -> UnboundedSender<ToQueryWorker> {
        self.req_tx.clone()
    }
}

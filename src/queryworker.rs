pub mod highlevelquery;
pub mod query;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::action::action::{Action, QueryAction};
use crate::config::Config;
use crate::lyricsclient::lrclib::LrcLib;
use crate::lyricsclient::LyricsClient;
use crate::osclient::response::empty::Empty;
use crate::osclient::response::getplaylist::{GetPlaylist, IndeterminedPlaylist};
use crate::osclient::response::getplaylists::GetPlaylists;
use crate::osclient::OSClient;
use crate::playerworker::player::ToPlayerWorker;
use crate::queryworker::highlevelquery::HighLevelQuery;
use crate::queryworker::query::getplaylist::GetPlaylistResponse;
use crate::queryworker::query::getplaylists::GetPlaylistsResponse;
use crate::queryworker::query::setcredential::Credential;
use crate::queryworker::query::{FromQueryWorker, ResponseType};
use crate::trace_dbg;
use color_eyre::Result;
use query::ToQueryWorker;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub struct QueryWorker {
    client: Option<Arc<OSClient>>,
    lyrics: Arc<LrcLib>,
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

    #[inline]
    fn send_action(&self, action: FromQueryWorker) {
        let _ = self
            .action_tx
            .send(Action::Query(QueryAction::FromQueryWorker(action)));
    }

    pub async fn run(&mut self) -> Result<()> {
        trace_dbg!("Starting QueryWorker...");
        loop {
            let Some(event) = self.req_rx.recv().await else {
                break;
            };
            match event.query {
                HighLevelQuery::SetStar { media, star } => {
                    let tx = self.action_tx.clone();
                    match &self.client {
                        Some(c) => {
                            let cc = c.clone();
                            tokio::spawn(async move {
                                let req = if star {
                                    cc.star(media.0).await
                                } else {
                                    cc.unstar(media.0).await
                                };
                                let result = match req {
                                    Ok(_) => ResponseType::Star(Ok(())),
                                    Err(err) => ResponseType::Star(Err(err.to_string())),
                                };
                                tx.send(Action::Query(QueryAction::FromQueryWorker(
                                    FromQueryWorker::new(event.dest, event.ticket, result),
                                )))
                            });
                        }
                        None => tracing::error!(
                            "Invalid state: Tried querying, but client does not exist!"
                        ),
                    };
                }
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
                            self.send_action(FromQueryWorker::new(
                                event.dest,
                                event.ticket,
                                ResponseType::SetCredential(Ok(())),
                            ));
                        }
                        Err(err) => {
                            self.send_action(FromQueryWorker::new(
                                event.dest,
                                event.ticket,
                                ResponseType::SetCredential(Err(err.to_string())),
                            ));
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
                                tx.send(Action::Query(QueryAction::FromQueryWorker(
                                    FromQueryWorker::new(
                                        event.dest,
                                        event.ticket,
                                        ResponseType::GetPlaylists(res),
                                    ),
                                )));
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
                                        Empty::Ok => tx.send(Action::Query(
                                            QueryAction::FromQueryWorker(FromQueryWorker::new(
                                                event.dest,
                                                event.ticket,
                                                ResponseType::Ping(Ok(())),
                                            )),
                                        )),
                                        Empty::Failed { error } => tx.send(Action::Query(
                                            QueryAction::FromQueryWorker(FromQueryWorker::new(
                                                event.dest,
                                                event.ticket,
                                                ResponseType::Ping(Err(error.to_string())),
                                            )),
                                        )),
                                    },
                                    Err(e) => tx.send(Action::Query(QueryAction::FromQueryWorker(
                                        FromQueryWorker::new(
                                            event.dest,
                                            event.ticket,
                                            ResponseType::Ping(Err(e.to_string())),
                                        ),
                                    ))),
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
                                                .send(Action::Query(QueryAction::FromQueryWorker(
                                                    FromQueryWorker::new(
                                                        event.dest,
                                                        event.ticket,
                                                        ResponseType::GetPlaylist(
                                                            GetPlaylistResponse::Success(
                                                                full_playlist,
                                                            ),
                                                        ),
                                                    ),
                                                ))),
                                            IndeterminedPlaylist::AmpacheEmpty(simple_playlist) => {
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
                                                tx.send(Action::Query(
                                                    QueryAction::FromQueryWorker(
                                                        FromQueryWorker::new(
                                                            event.dest,
                                                            event.ticket,
                                                            ResponseType::GetPlaylist(res),
                                                        ),
                                                    ),
                                                ))
                                            }
                                            IndeterminedPlaylist::NavidromeEmpty(
                                                simple_playlist,
                                            ) => tx.send(Action::Query(
                                                QueryAction::FromQueryWorker(FromQueryWorker::new(
                                                    event.dest,
                                                    event.ticket,
                                                    ResponseType::GetPlaylist(
                                                        GetPlaylistResponse::Partial(
                                                            simple_playlist,
                                                        ),
                                                    ),
                                                )),
                                            )),
                                        },

                                        GetPlaylist::Failed { error } => tx.send(Action::Query(
                                            QueryAction::FromQueryWorker(FromQueryWorker::new(
                                                event.dest,
                                                event.ticket,
                                                ResponseType::GetPlaylist(
                                                    GetPlaylistResponse::Failure {
                                                        id: params.id,
                                                        name: params.name,
                                                        msg: error.to_string(),
                                                    },
                                                ),
                                            )),
                                        )),
                                    },
                                    Err(e) => tx.send(Action::Query(QueryAction::FromQueryWorker(
                                        FromQueryWorker::new(
                                            event.dest,
                                            event.ticket,
                                            ResponseType::GetPlaylist(
                                                GetPlaylistResponse::Failure {
                                                    id: params.id,
                                                    name: params.name,
                                                    msg: e.to_string(),
                                                },
                                            ),
                                        ),
                                    ))),
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
                            let url = c.stream_link(id.0).to_string();
                            let _ = self.action_tx.send(Action::ToPlayerWorker(
                                ToPlayerWorker::PlayURL { music: media, url },
                            ));
                        }
                        None => tracing::error!(
                            "Invalid state: Tried querying, but client does not exist!"
                        ),
                    };
                }
                HighLevelQuery::GetLyrics(params) => {
                    let c = self.lyrics.clone();
                    let tx = self.action_tx.clone();
                    tokio::spawn(async move {
                        let res = FromQueryWorker::new(
                            event.dest,
                            event.ticket,
                            ResponseType::GetLyrics(match c.search(params).await {
                                Ok(success) => Ok(success),
                                Err(failed) => Err(failed.to_string()),
                            }),
                        );
                        let _ = tx.send(Action::Query(QueryAction::FromQueryWorker(res)));
                    });
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
            lyrics: Arc::new(LrcLib::new(config.clone())),
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

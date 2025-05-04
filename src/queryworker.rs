pub mod query;

use std::sync::Arc;

use color_eyre::{eyre, Result};
use query::setcredential::Credential;
use query::Query;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::action::getplaylist::{FullPlaylist, GetPlaylistResponse, PlaylistEntry};
use crate::action::getplaylists::{GetPlaylistsResponse, SimplePlaylist};
use crate::action::ping::PingResponse;
use crate::action::Action;
use crate::config::Config;
use crate::osclient::response::empty::Empty;
use crate::osclient::response::getplaylist::GetPlaylist;
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
    fn wrapper(&self, cb: impl Fn(Arc<OSClient>, UnboundedSender<Action>) -> ()) {
        match &self.client {
            Some(c) => {
                let tx = self.action_tx.clone();
                cb(c.clone(), tx);
            }
            None => tracing::error!("Invalid state: Tried querying, but client does not exist!"),
        };
    }

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
                Query::GetPlaylist(id) => {
                    let idc = id.clone();
                    match &self.client {
                        Some(c) => {
                            let tx = self.action_tx.clone();
                            let client = c.clone();
                            tokio::spawn(async move {
                                let res = client.get_playlist(idc).await;
                                match res {
                                    Ok(c) => match c {
                                        GetPlaylist::Ok { playlist } => {
                                            tx.send(Action::GetPlaylist(
                                                GetPlaylistResponse::Success(FullPlaylist {
                                                    id: playlist.id,
                                                    name: playlist.name,
                                                    owner: playlist.owner,
                                                    public: playlist.public,
                                                    created: playlist.created,
                                                    changed: playlist.changed,
                                                    song_count: playlist.song_count,
                                                    duration: playlist.duration,
                                                    entry: playlist
                                                        .entry
                                                        .into_iter()
                                                        .map(|e| PlaylistEntry {
                                                            id: e.id,
                                                            parent: e.parent,
                                                            title: e.title,
                                                            is_dir: e.is_dir,
                                                            is_video: e.is_video,
                                                            entry_type: e.entry_type,
                                                            album_id: e.album_id,
                                                            album: e.album,
                                                            artist_id: e.artist_id,
                                                            artist: e.artist,
                                                            cover_art: e.cover_art,
                                                            duration: e.duration,
                                                            bit_rate: e.bit_rate,
                                                            bit_depth: e.bit_depth,
                                                            sampling_rate: e.sampling_rate,
                                                            channel_count: e.channel_count,
                                                            user_rating: e.user_rating,
                                                            average_rating: e.average_rating,
                                                            track: e.track,
                                                            year: e.year,
                                                            genre: e.genre,
                                                            size: e.size,
                                                            disc_number: e.disc_number,
                                                            suffix: e.suffix,
                                                            content_type: e.content_type,
                                                            path: e.path,
                                                        })
                                                        .collect(),
                                                }),
                                            ))
                                        }

                                        GetPlaylist::Failed { error } => {
                                            tx.send(Action::GetPlaylist(
                                                GetPlaylistResponse::Failure(error.to_string()),
                                            ))
                                        }
                                    },
                                    Err(e) => tx.send(Action::GetPlaylist(
                                        GetPlaylistResponse::Failure(e.to_string()),
                                    )),
                                }
                            });
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
    pub fn get_tx(&self) -> UnboundedSender<Query> {
        self.req_tx.clone()
    }
}

pub mod query;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::action::Action;
use crate::config::Config;
use crate::osclient::response::empty::Empty;
use crate::osclient::response::getplaylist::{FullPlaylist, GetPlaylist, Media};
use crate::osclient::response::getplaylists::{GetPlaylists, SimplePlaylist};
use crate::osclient::OSClient;
use crate::playerworker::player::ToPlayerWorker;
use crate::queryworker::query::getplaylist::GetPlaylistResponse;
use crate::queryworker::query::getplaylists::GetPlaylistsResponse;
use crate::queryworker::query::ping::PingResponse;
use crate::queryworker::query::setcredential::Credential;
use crate::queryworker::query::{FromQueryWorker, QueryType, ResponseType};
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
            match event.query {
                QueryType::SetCredential(creds) => {
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
                QueryType::GetPlaylists => {
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
                QueryType::Ping => {
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
                QueryType::GetPlaylist { name, id } => {
                    let idc = id.clone();
                    match &self.client {
                        Some(c) => {
                            let tx = self.action_tx.clone();
                            let client = c.clone();
                            tokio::spawn(async move {
                                let res = client.get_playlist(String::from(idc)).await;
                                match res {
                                    Ok(c) => match c {
                                        GetPlaylist::Ok { playlist } => {
                                            tx.send(Action::FromQueryWorker(FromQueryWorker::new(
                                                event.dest,
                                                event.ticket,
                                                ResponseType::GetPlaylist(
                                                    GetPlaylistResponse::Success(FullPlaylist {
                                                        entry: playlist
                                                            .entry
                                                            .into_iter()
                                                            .map(|e| Media {
                                                                id: e.id,
                                                                parent: e.parent,
                                                                is_dir: e.is_dir,
                                                                title: e.title,
                                                                album: e.album,
                                                                artist: e.artist,
                                                                track: e.track,
                                                                year: e.year,
                                                                genre: e.genre,
                                                                cover_art: e.cover_art,
                                                                size: e.size,
                                                                content_type: e.content_type,
                                                                suffix: e.suffix,
                                                                transcoded_content_type: e
                                                                    .transcoded_content_type,
                                                                transcoded_suffix: e
                                                                    .transcoded_suffix,
                                                                duration: e.duration,
                                                                bit_rate: e.bit_rate,
                                                                bit_depth: e.bit_depth,
                                                                sampling_rate: e.sampling_rate,
                                                                channel_count: e.channel_count,
                                                                path: e.path,
                                                                is_video: e.is_video,
                                                                user_rating: e.user_rating,
                                                                average_rating: e.average_rating,
                                                                play_count: e.play_count,
                                                                disc_number: e.disc_number,
                                                                created: e.created,
                                                                starred: e.starred,
                                                                album_id: e.album_id,
                                                                artist_id: e.artist_id,
                                                                media_type: e.media_type,
                                                                bookmark_position: e
                                                                    .bookmark_position,
                                                                original_width: e.original_width,
                                                                original_height: e.original_height,
                                                                played: e.played,
                                                                bpm: e.bpm,
                                                                comment: e.comment,
                                                                sort_name: e.sort_name,
                                                                music_brainz_id: e.music_brainz_id,
                                                                display_artist: e.display_artist,
                                                                display_album_artist: e
                                                                    .display_album_artist,
                                                                display_composer: e
                                                                    .display_composer,
                                                                moods: e.moods,
                                                                explicit_status: e.explicit_status,
                                                            })
                                                            .collect(),
                                                        id,
                                                        name: playlist.name,
                                                        comment: playlist.comment,
                                                        owner: playlist.owner,
                                                        public: playlist.public,
                                                        song_count: playlist.song_count,
                                                        duration: playlist.duration,
                                                        created: playlist.created,
                                                        changed: playlist.changed,
                                                        cover_art: playlist.cover_art,
                                                        allowed_users: playlist.allowed_users,
                                                    }),
                                                ),
                                            )))
                                        }

                                        GetPlaylist::Failed { error } => {
                                            tx.send(Action::FromQueryWorker(FromQueryWorker::new(
                                                event.dest,
                                                event.ticket,
                                                ResponseType::GetPlaylist(
                                                    GetPlaylistResponse::Failure {
                                                        id,
                                                        msg: error.to_string(),
                                                        name,
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
                                                    id,
                                                    msg: e.to_string(),
                                                    name,
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
                QueryType::GetUrlByMedia { media } => {
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

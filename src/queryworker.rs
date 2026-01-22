pub mod highlevelquery;
pub mod query;

use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::action::action::Action;
use crate::compid::CompID;
use crate::config::Config;
use crate::lyricsclient::getlyrics::GetLyricsParams;
use crate::lyricsclient::lrclib::LrcLib;
use crate::lyricsclient::LyricsClient;
use crate::osclient::response::empty::Empty;
use crate::osclient::response::getplaylist::{GetPlaylist, IndeterminedPlaylist, Media};
use crate::osclient::response::getplaylists::{GetPlaylists, SimplePlaylist};
use crate::osclient::types::CoverID;
use crate::osclient::OSClient;
use crate::playerworker::player::ToPlayerWorker;
use crate::queryworker::highlevelquery::HighLevelQuery;
use crate::queryworker::query::getplaylist::GetPlaylistResponse;
use crate::queryworker::query::setcredential::Credential;
use crate::queryworker::query::{QueryStatus, ResponseType};
use crate::trace_dbg;
use bytes::Bytes;
use color_eyre::Result;
use image::{DynamicImage, ImageReader};
use query::ToQueryWorker;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;

#[derive(Default)]
struct Cache {
    pub playlists: Option<Vec<SimplePlaylist>>,
}

struct AwaitingQueries {
    play_from_url: Option<(Media, u16)>,
    get_cover: Option<(Vec<CompID>, usize, CoverID, u16)>,
    get_lyrics: Option<(Vec<CompID>, usize, GetLyricsParams, u16)>,
}

impl AwaitingQueries {
    pub fn register_get_lyrics(
        &mut self,
        compid: Vec<CompID>,
        ticket: usize,
        params: GetLyricsParams,
    ) -> Option<usize> {
        let ret = if let Some((_, ticket, _, _)) = self.get_lyrics {
            Some(ticket)
        } else {
            None
        };
        self.get_lyrics = Some((compid, ticket, params, 2));
        ret
    }
    pub fn register_play_from_url(&mut self, media: Media) {
        self.play_from_url = Some((media, 2));
    }
    pub fn register_get_cover(
        &mut self,
        compid: Vec<CompID>,
        ticket: usize,
        id: CoverID,
    ) -> Option<usize> {
        let ret = if let Some((_, ticket, _, _)) = self.get_cover {
            Some(ticket)
        } else {
            None
        };
        self.get_cover = Some((compid, ticket, id, 2));
        ret
    }
}

pub struct QueryWorker {
    client: Option<Arc<OSClient>>,
    lyrics: Arc<LrcLib>,
    req_tx: UnboundedSender<ToQueryWorker>,
    req_rx: UnboundedReceiver<ToQueryWorker>,
    action_tx: UnboundedSender<Action>,
    should_quit: bool,
    awaiting: AwaitingQueries,

    cache: Arc<Mutex<Cache>>,
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

impl QueryWorker {
    /// Returns a unique ticket number
    /// This value must be included in every request sent to this worker
    pub fn get_ticket() -> usize {
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    pub fn on_tick(&mut self) {
        macro_rules! process_awaiting {
            ($self:ident, $item:ident, $func:ident) => {
                let item = if let Some((compid, ticket, id, remaining)) = &mut $self.awaiting.$item
                {
                    if *remaining == 0 {
                        Some((compid.clone(), *ticket, id.clone()))
                    } else {
                        *remaining -= 1;
                        None
                    }
                } else {
                    None
                };
                if let Some((compid, ticket, id)) = item {
                    $self.awaiting.$item = None;
                    $self.$func(compid, ticket, id);
                }
            };
        }
        process_awaiting!(self, get_cover, get_cover);
        process_awaiting!(self, get_lyrics, get_lyrics);

        let play = if let Some((m, remaining)) = &mut self.awaiting.play_from_url {
            if *remaining == 0 {
                Some(m.clone())
            } else {
                *remaining -= 1;
                None
            }
        } else {
            None
        };
        if let Some(m) = play {
            self.awaiting.play_from_url = None;
            self.play_from_url(m);
        }
    }

    pub fn get_playlists(&self, event: ToQueryWorker, force_query: bool) {
        if !force_query {
            if let Ok(lock) = self.cache.try_lock() {
                if let Some(playlist) = &lock.playlists {
                    let _ = self.action_tx.send(Action::FromQuery {
                        dest: event.dest,
                        ticket: event.ticket,
                        res: QueryStatus::Finished(ResponseType::GetPlaylists(
                            Ok(playlist.clone()),
                        )),
                    });
                    return;
                }
            }
        }
        let (tx, c) = self.prepare_async();
        let cache = self.cache.clone();
        tokio::spawn(async move {
            let res = match c.get_playlists().await {
                Ok(r) => match r {
                    GetPlaylists::Ok { playlists } => {
                        if let Ok(mut lock) = cache.try_lock() {
                            lock.playlists = Some(playlists.playlist.clone());
                        }
                        Ok(playlists.playlist)
                    }
                    GetPlaylists::Failed { error } => Err(error.to_string()),
                },
                Err(e) => Err(e.to_string()),
            };
            let _ = tx.send(Action::FromQuery {
                dest: event.dest,
                ticket: event.ticket,
                res: QueryStatus::Finished(ResponseType::GetPlaylists(res)),
            });
        });
    }

    fn prepare_async(&self) -> (UnboundedSender<Action>, Arc<OSClient>) {
        match &self.client {
            Some(client) => (self.action_tx.clone(), client.clone()),
            None => panic!("Invalid state: Tried querying, but client does not exist!"),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        trace_dbg!("Starting QueryWorker...");
        loop {
            let Some(event) = self.req_rx.recv().await else {
                break;
            };
            match event.query {
                HighLevelQuery::SetStar { media, star } => {
                    let (tx, c) = self.prepare_async();
                    tokio::spawn(async move {
                        let req = if star {
                            c.star(media).await
                        } else {
                            c.unstar(media).await
                        };
                        let result = match req {
                            Ok(_) => ResponseType::Star(Ok(())),
                            Err(err) => ResponseType::Star(Err(err.to_string())),
                        };
                        tx.send(Action::FromQuery {
                            dest: event.dest,
                            ticket: event.ticket,
                            res: QueryStatus::Finished(result),
                        })
                    });
                }
                HighLevelQuery::ListPlaylists => self.get_playlists(event, true),
                HighLevelQuery::ListPlaylistsPopup(force) => self.get_playlists(event, force),
                HighLevelQuery::Login(creds) => {
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
                            let (tx, c) = self.prepare_async();
                            tokio::spawn(async move {
                                let ping = c.ping().await;
                                match ping {
                                    Ok(c) => match c {
                                        Empty::Ok => tx.send(Action::FromQuery {
                                            dest: event.dest,
                                            ticket: event.ticket,
                                            res: QueryStatus::Finished(ResponseType::Login(Ok(()))),
                                        }),
                                        Empty::Failed { error } => tx.send(Action::FromQuery {
                                            dest: event.dest,
                                            ticket: event.ticket,
                                            res: QueryStatus::Finished(ResponseType::Login(Err(
                                                error.to_string(),
                                            ))),
                                        }),
                                    },
                                    Err(e) => tx.send(Action::FromQuery {
                                        dest: event.dest,
                                        ticket: event.ticket,
                                        res: QueryStatus::Finished(ResponseType::Login(Err(
                                            e.to_string()
                                        ))),
                                    }),
                                }
                            });
                        }
                        Err(err) => {
                            self.action_tx.send(Action::FromQuery {
                                dest: event.dest,
                                ticket: event.ticket,
                                res: QueryStatus::Finished(ResponseType::Login(Err(
                                    err.to_string()
                                ))),
                            })?;
                        }
                    };
                }
                HighLevelQuery::SelectPlaylist(params)
                | HighLevelQuery::AddPlaylistToQueue(params) => {
                    let (tx, c) = self.prepare_async();
                    tokio::spawn(async move {
                        let res = c.get_playlist(params.id.clone()).await;
                        match res {
                            Ok(c) => match c {
                                GetPlaylist::Ok { playlist } => match playlist {
                                    IndeterminedPlaylist::FullPlaylist(full_playlist) => {
                                        let _ = tx.send(Action::FromQuery {
                                            dest: event.dest,
                                            ticket: event.ticket,
                                            res: QueryStatus::Finished(ResponseType::GetPlaylist(
                                                GetPlaylistResponse::Success(full_playlist),
                                            )),
                                        });
                                    }
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
                                        let _ = tx.send(Action::FromQuery {
                                            dest: event.dest,
                                            ticket: event.ticket,
                                            res: QueryStatus::Finished(ResponseType::GetPlaylist(
                                                res,
                                            )),
                                        });
                                    }
                                    IndeterminedPlaylist::NavidromeEmpty(simple_playlist) => {
                                        let _ = tx.send(Action::FromQuery {
                                            dest: event.dest,
                                            ticket: event.ticket,
                                            res: QueryStatus::Finished(ResponseType::GetPlaylist(
                                                GetPlaylistResponse::Partial(simple_playlist),
                                            )),
                                        });
                                    }
                                },

                                GetPlaylist::Failed { error } => {
                                    let _ = tx.send(Action::FromQuery {
                                        dest: event.dest,
                                        ticket: event.ticket,
                                        res: QueryStatus::Finished(ResponseType::GetPlaylist(
                                            GetPlaylistResponse::Failure {
                                                id: params.id,
                                                name: params.name,
                                                msg: error.to_string(),
                                            },
                                        )),
                                    });
                                }
                            },
                            Err(e) => {
                                let _ = tx.send(Action::FromQuery {
                                    dest: event.dest,
                                    ticket: event.ticket,
                                    res: QueryStatus::Finished(ResponseType::GetPlaylist(
                                        GetPlaylistResponse::Failure {
                                            id: params.id,
                                            name: params.name,
                                            msg: e.to_string(),
                                        },
                                    )),
                                });
                            }
                        }
                    });
                }
                HighLevelQuery::PlayMusicFromURL(media) => {
                    self.awaiting.register_play_from_url(media);
                }
                HighLevelQuery::GetLyrics(params) => {
                    if let Some(prev) =
                        self.awaiting
                            .register_get_lyrics(event.dest.clone(), event.ticket, params)
                    {
                        let _ = self.action_tx.send(Action::FromQuery {
                            dest: event.dest,
                            ticket: prev,
                            res: QueryStatus::Aborted(true),
                        });
                    };
                }
                HighLevelQuery::GetCover(cover_id) => {
                    if let Some(prev) =
                        self.awaiting
                            .register_get_cover(event.dest.clone(), event.ticket, cover_id)
                    {
                        let _ = self.action_tx.send(Action::FromQuery {
                            dest: event.dest,
                            ticket: prev,
                            res: QueryStatus::Aborted(true),
                        });
                    }
                }
                HighLevelQuery::Tick => self.on_tick(),
                HighLevelQuery::UpdatePlaylist(update_playlist_params) => {
                    let (tx, c) = self.prepare_async();
                    tokio::spawn(async move {
                        let res = c
                            .update_playlist(
                                update_playlist_params.playlist_id,
                                update_playlist_params.name,
                                update_playlist_params.comment,
                                update_playlist_params.public,
                                update_playlist_params.song_id_to_add,
                                update_playlist_params.song_index_to_remove,
                            )
                            .await;
                        let _ = tx.send(Action::FromQuery {
                            dest: event.dest,
                            ticket: event.ticket,
                            res: QueryStatus::Finished(ResponseType::UpdatePlaylist(match res {
                                Ok(_) => Ok(()),
                                Err(e) => Err(e.to_string()),
                            })),
                        });
                    });
                }
            };
            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn get_lyrics(&mut self, dest: Vec<CompID>, ticket: usize, params: GetLyricsParams) {
        let c = self.lyrics.clone();
        let tx = self.action_tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(Action::FromQuery {
                dest: dest,
                ticket: ticket,
                res: QueryStatus::Finished(ResponseType::GetLyrics(match c.search(params).await {
                    Ok(success) => Ok(success),
                    Err(failed) => Err(failed.to_string()),
                })),
            });
        });
    }

    fn get_cover(&mut self, dest: Vec<CompID>, ticket: usize, cover_id: CoverID) {
        let (tx, c) = self.prepare_async();
        tokio::spawn(async move {
            let art = c.get_cover_art(cover_id.0).await;
            match art {
                Ok(c) => match c {
                    Err(e) => {
                        let _ = tx.send(Action::FromQuery {
                            dest,
                            ticket,
                            res: QueryStatus::Finished(ResponseType::GetCover(Err(e
                                .error
                                .to_string()))),
                        });
                    }
                    Ok(b) => {
                        fn decode_image(b: Bytes) -> Result<DynamicImage, String> {
                            let Ok(reader) = ImageReader::new(Cursor::new(b)).with_guessed_format()
                            else {
                                return Err("Failed to determine image format!".to_string());
                            };
                            let Ok(decoded) = reader.decode() else {
                                return Err("Failed to decode image!".to_string());
                            };
                            Ok(decoded)
                        }
                        let _ = tx.send(Action::FromQuery {
                            dest,
                            ticket,
                            res: QueryStatus::Finished(ResponseType::GetCover(decode_image(b))),
                        });
                    }
                },
                Err(e) => {
                    let _ = tx.send(Action::FromQuery {
                        dest,
                        ticket,
                        res: QueryStatus::Finished(ResponseType::GetCover(Err(e.to_string()))),
                    });
                }
            }
        });
    }

    fn play_from_url(&mut self, media: Media) {
        match &self.client {
            Some(c) => {
                let id = media.id.clone();
                let url = c.stream_link(id).to_string();
                let _ = self
                    .action_tx
                    .send(Action::ToPlayer(ToPlayerWorker::PlayURL {
                        music: media,
                        url,
                    }));
            }
            None => tracing::error!("Invalid state: Tried querying, but client does not exist!"),
        };
    }
}

impl QueryWorker {
    pub fn new(sender: UnboundedSender<Action>, config: Config) -> Self {
        let (req_tx, req_rx) = mpsc::unbounded_channel();
        let awaiting = AwaitingQueries {
            play_from_url: None,
            get_cover: None,
            get_lyrics: None,
        };
        Self {
            lyrics: Arc::new(LrcLib::new(config.clone())),
            client: None,
            req_tx,
            req_rx,
            action_tx: sender,
            should_quit: false,
            awaiting,

            cache: Arc::new(Mutex::new(Cache::default())),
        }
    }
    pub fn get_tx(&self) -> UnboundedSender<ToQueryWorker> {
        self.req_tx.clone()
    }
}

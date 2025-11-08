use reqwest::{Client, Method, Url};
use serde_json::from_str;

use crate::{
    config::Config,
    lyricsclient::{
        getlyrics::{GetLyricsParams, GetLyricsResponse},
        FailReason, LyricsClient,
    },
};

pub struct LrcLib {
    client: Client,
    config: Config,
}

impl LrcLib {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            client: Client::builder()
                .build()
                .expect("Failed to create reqwest client for LrcLib."),
        }
    }
}

impl LyricsClient for LrcLib {
    /// Given lyrics parameters, search for a music with lyrics.
    /// It uses /search
    async fn search(
        &self,
        params: GetLyricsParams,
    ) -> Result<Option<GetLyricsResponse>, FailReason> {
        let Ok(mut url) = Url::parse(&self.config.config.lrc_url) else {
            return Err(FailReason::URLParsing);
        };
        url.set_path("api/search");
        url.query_pairs_mut()
            .append_pair("track_name", &params.track_name);
        // if let Some(an) = params.artist_name {
        //     url.query_pairs_mut().append_pair("artist_name", &an);
        // }
        // if let Some(an) = params.album_name {
        //     url.query_pairs_mut().append_pair("album_name", &an);
        // }
        let res = match self.client.request(Method::GET, url).send().await {
            Ok(res) => res,
            Err(err) => {
                return Err(FailReason::Querying(err));
            }
        };
        match res.error_for_status() {
            Ok(r) => {
                let Ok(body) = r.text().await else {
                    return Err(FailReason::Text);
                };
                let Ok(data) = from_str::<Vec<GetLyricsResponse>>(&body) else {
                    return Err(FailReason::Decoding);
                };
                if data.len() == 0 {
                    return Ok(None);
                }
                let synced = data
                    .iter()
                    .find(|item| matches!(item.synced_lyrics, Some(_)));
                if let Some(found) = synced {
                    return Ok(Some(found.clone()));
                }
                let plain = data
                    .iter()
                    .find(|item| matches!(item.plain_lyrics, Some(_)));
                if let Some(found) = plain {
                    return Ok(Some(found.clone()));
                } else {
                    return Ok(None);
                }
            }
            Err(status) => {
                if let Some(code) = status.status() {
                    return Err(FailReason::ErrStatus(code));
                } else {
                    panic!("Received error status code but found no error status code??")
                }
            }
        };
    }
}

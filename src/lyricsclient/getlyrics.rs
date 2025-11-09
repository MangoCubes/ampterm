use std::time::Duration;

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct GetLyricsParams {
    pub track_name: String,
    pub artist_name: Option<String>,
    pub album_name: Option<String>,
    pub length: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct GetLyricsResponse {
    pub id: u32,
    pub name: String,
    #[serde(alias = "trackName")]
    pub track_name: String,
    #[serde(alias = "artistName")]
    pub artist_name: String,
    #[serde(alias = "albumName")]
    pub album_name: String,
    pub duration: f64,
    pub instrumental: bool,
    #[serde(alias = "plainLyrics")]
    pub plain_lyrics: Option<String>,
    #[serde(alias = "syncedLyrics")]
    pub synced_lyrics: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LyricLine {
    pub time: Duration,
    pub lyric: String,
}

pub struct ParsedLyrics {
    pub lyrics: Vec<LyricLine>,
}

impl ParsedLyrics {
    /// Parses received response into more program-friendly format
    /// Expects the response to come in this following format:
    ///  [01:19.08] 「あなた段々眠くなる」浅はかな催眠術
    /// 1 -- Minute
    /// 2    -- Seconds
    /// 3       -- Milisecs
    /// 4           -----------... Lyrics until the end
    ///
    pub fn from(raw: String) -> Self {
        let regex = Regex::new(r"^\[(\d+):(\d+)\.(\d+)\] (.*)$").unwrap();
        let lyrics = raw
            .split('\n')
            .filter_map(|s| {
                let capture = regex.captures(s)?;
                let min: u64 = capture.get(1)?.as_str().parse().ok()?;
                let sec: u64 = capture.get(2)?.as_str().parse().ok()?;
                let milisecs: u64 = capture.get(3)?.as_str().parse().ok()?;
                let lyric = {
                    let orig = capture.get(4)?.as_str().to_string();
                    if orig.len() == 0 {
                        "𝆺𝅥𝅮𝆺𝅥𝅮𝆺𝅥𝅮𝆺𝅥𝅮".to_string()
                    } else {
                        orig
                    }
                };
                let time =
                    Duration::from_millis((min * 60 * 1000) + (sec * 1000) + (milisecs * 10));
                Some(LyricLine { time, lyric })
            })
            .collect();
        ParsedLyrics { lyrics }
    }

    pub fn get_lyrics(
        &self,
        now: Duration,
    ) -> (Option<LyricLine>, Option<LyricLine>, Option<LyricLine>) {
        let len = self.lyrics.len();
        // No lyrics
        if len == 0 {
            return (None, None, None);
        }
        let current = &self.lyrics[0];
        // Before the first line
        if current.time >= now {
            return (None, None, Some(current.clone()));
        } else {
            // Edge case where there is only one lyric line
            if len == 1 {
                return (None, Some(current.clone()), None);
            }
        }
        let next = &self.lyrics[1];
        if next.time >= now {
            return (None, Some(current.clone()), Some(next.clone()));
        }

        for i in 2..len {
            let prev = self.lyrics.get(i - 2);
            let current = self.lyrics.get(i - 1);
            let next = self
                .lyrics
                .get(i)
                .expect("Lyrics for this index should exist, but for some reason it doesn't...");
            if next.time >= now {
                return (prev.cloned(), current.cloned(), Some(next.clone()));
            }
        }
        (
            self.lyrics.get(len - 2).cloned(),
            self.lyrics.get(len - 1).cloned(),
            None,
        )
    }
}

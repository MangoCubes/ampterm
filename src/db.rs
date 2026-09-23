use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection, Error as SqlError, Result as SqlResult};

use crate::lyricsclient::getlyrics::{GetLyricsParams, GetLyricsResponse};

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(path: &Path) -> SqlResult<Self> {
        if !path.exists() {
            fs::create_dir_all(path)
                .expect("Failed to create the directory to store the cache database.");
        }
        let db = Self {
            conn: Arc::new(Mutex::new(Connection::open(
                path.join(format!("{}.db", env!("CARGO_PKG_NAME"))),
            )?)),
        };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
            CREATE TABLE IF NOT EXISTS lyrics (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                track_name TEXT NOT NULL,
                artist_name TEXT NOT NULL,
                album_name TEXT NOT NULL,
                duration INTEGER NOT NULL,
                instrumental INTEGER NOT NULL,
                plain_lyrics TEXT,
                synced_lyrics TEXT
            );",
        )?;
        Ok(())
    }

    pub fn get_lyrics(&self, params: &GetLyricsParams) -> SqlResult<Option<GetLyricsResponse>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, track_name, artist_name, album_name, duration, instrumental, plain_lyrics, synced_lyrics FROM lyrics WHERE track_name = ?")?;

        let results = stmt
            .query_map(params![&params.track_name], |row| {
                Ok(GetLyricsResponse {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    track_name: row.get(2)?,
                    artist_name: row.get(3)?,
                    album_name: row.get(4)?,
                    duration: row.get(5)?,
                    instrumental: row.get::<usize, i32>(6)? != 0,
                    plain_lyrics: row.get(7)?,
                    synced_lyrics: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<GetLyricsResponse>, SqlError>>()?;

        Ok(results.into_iter().find(|item| {
            params
                .length
                .map_or(false, |len| ((item.duration as i32) - len).abs() < 3)
        }))
    }

    pub fn save_lyrics(
        &self,
        _params: &GetLyricsParams,
        response: &GetLyricsResponse,
    ) -> SqlResult<()> {
        self.conn.lock().unwrap().execute(
            "INSERT OR REPLACE INTO lyrics VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                response.id,
                response.name,
                response.track_name,
                response.artist_name,
                response.album_name,
                response.duration,
                response.instrumental,
                response.plain_lyrics,
                response.synced_lyrics,
            ],
        )?;
        Ok(())
    }
}

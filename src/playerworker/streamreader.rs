use std::time::Instant;

use reqwest::Url;
use stream_download::http::HttpStream;
use stream_download::source::SourceStream;
use stream_download::storage::temp::TempStorageProvider;
use stream_download::{Settings, StreamDownload, StreamPhase};
use tokio::sync::mpsc::UnboundedSender;

use crate::action::{Action, FromPlayerWorker};

use super::streamerror::StreamError;

pub struct StreamReader {}

impl StreamReader {
    pub async fn get_reader(
        url: Url,
        action_tx: UnboundedSender<Action>,
    ) -> Result<StreamDownload<TempStorageProvider>, StreamError> {
        let mut last_event = Instant::now();
        let settings = Settings::default().on_progress(move |stream: &HttpStream<_>, state, _| {
            let now = Instant::now();
            let elapsed = now - last_event;
            last_event = now;
            let msg = match state.phase {
                StreamPhase::Prefetching {
                    target, chunk_size, ..
                } => {
                    format!(
                        "{:.2?} prefetch progress: {:.2}% downloaded: {:?} kb/s: {:.2}",
                        state.elapsed,
                        (state.current_position as f32 / target as f32) * 100.0,
                        state.current_chunk,
                        chunk_size as f32 / elapsed.as_nanos() as f32 * 1000.0
                    )
                }
                StreamPhase::Downloading { chunk_size, .. } => {
                    format!(
                        "{:.2?} download progress {:.2}% downloaded: {:?} kb/s: {:.2}",
                        state.elapsed,
                        (state.current_position as f32 / stream.content_length().unwrap() as f32)
                            * 100.0,
                        state.current_chunk,
                        chunk_size as f32 / elapsed.as_nanos() as f32 * 1000.0
                    )
                }
                StreamPhase::Complete => {
                    format!("{:.2?} download complete", state.elapsed)
                }
                _ => String::default(),
            };
            action_tx.send(Action::FromPlayerWorker(FromPlayerWorker::Message(msg)));
        });
        match StreamDownload::new_http(url, TempStorageProvider::new(), settings).await {
            Ok(reader) => Ok(reader),
            Err(e) => Err(StreamError::stream_init(e)),
        }
    }
}

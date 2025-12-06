use serde::Deserialize;

use crate::config::lyricsconfig::LyricsConfig;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FeaturesConfig {
    pub lyrics: LyricsConfig,
}

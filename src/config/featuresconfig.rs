use serde::Deserialize;

use crate::config::{
    bpmtoyconfig::BPMToyConfig, coverartconfig::CoverArtConfig, lyricsconfig::LyricsConfig,
};

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FeaturesConfig {
    #[serde(default)]
    pub lyrics: LyricsConfig,
    #[serde(default)]
    pub bpmtoy: BPMToyConfig,
    #[serde(default)]
    pub cover_art: CoverArtConfig,
}

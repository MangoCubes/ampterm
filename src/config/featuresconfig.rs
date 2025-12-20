use serde::Deserialize;

use crate::config::{
    bpmtoyconfig::BPMToyConfig, coverartconfig::CoverArtConfig, lyricsconfig::LyricsConfig,
};

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FeaturesConfig {
    pub lyrics: LyricsConfig,
    pub bpmtoy: BPMToyConfig,
    pub cover_art: CoverArtConfig,
}

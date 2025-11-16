use serde::Deserialize;
#[derive(Clone, Debug, Deserialize, Default)]
pub struct LyricsConfig {
    #[serde(default)]
    pub enable: bool,
}

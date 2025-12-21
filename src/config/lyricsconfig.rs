use serde::Deserialize;

fn default_true() -> bool {
    true
}

fn default_lrclib() -> String {
    "https://lrclib.net".to_string()
}

#[derive(Clone, Debug, Deserialize)]
pub struct LyricsConfig {
    #[serde(default = "default_true")]
    pub enable: bool,
    #[serde(default = "default_lrclib")]
    pub lrc_url: String,
}

impl Default for LyricsConfig {
    fn default() -> Self {
        Self {
            enable: true,
            lrc_url: "https://lrclib.net".to_string(),
        }
    }
}

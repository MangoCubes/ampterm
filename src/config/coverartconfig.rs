use serde::Deserialize;

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct CoverArtConfig {
    #[serde(default = "default_true")]
    pub enable: bool,
}

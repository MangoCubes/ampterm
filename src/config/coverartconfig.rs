use serde::Deserialize;

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize)]
pub struct CoverArtConfig {
    #[serde(default = "default_true")]
    pub enable: bool,
}
impl Default for CoverArtConfig {
    fn default() -> Self {
        Self { enable: true }
    }
}

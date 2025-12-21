use serde::Deserialize;

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize)]
pub struct BPMToyConfig {
    #[serde(default = "default_true")]
    pub enable: bool,
}

impl Default for BPMToyConfig {
    fn default() -> Self {
        Self { enable: true }
    }
}

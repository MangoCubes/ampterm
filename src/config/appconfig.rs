use std::path::PathBuf;

use serde::Deserialize;

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub data_dir: PathBuf,
    #[serde(default)]
    pub config_dir: PathBuf,
    #[serde(default)]
    pub use_legacy_auth: bool,
    #[serde(default = "default_true")]
    pub auto_focus: bool,
}

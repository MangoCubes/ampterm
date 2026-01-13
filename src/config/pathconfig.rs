use std::{env, path::PathBuf};

use directories::ProjectDirs;
use lazy_static::lazy_static;
use stream_download::storage::temp::tempfile::TempDir;
lazy_static! {
    pub static ref PROJECT_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
    pub static ref DATA_FOLDER: Option<PathBuf> =
        env::var(format!("{}_DATA", PROJECT_NAME.clone()))
            .ok()
            .map(PathBuf::from);
    pub static ref CONFIG_FOLDER: Option<PathBuf> =
        env::var(format!("{}_CONFIG", PROJECT_NAME.clone()))
            .ok()
            .map(PathBuf::from);
}

pub struct PathConfig {
    pub data: PathBuf,
    pub config: Option<PathBuf>,
}

pub enum PathType {
    Custom(String),
    Default,
    None,
}

/// Path priority:
/// 1. Path specified via --data or --config
/// 2. Environment variable set via AMPTERM_DATA or AMPTERM_CONFIG
/// 3. XDG paths
/// 4. ./.data and ./.config
impl PathConfig {
    pub fn get_data_dir() -> PathBuf {
        if let Some(s) = DATA_FOLDER.clone() {
            s
        } else if let Some(proj_dirs) = Self::project_directory() {
            proj_dirs.data_local_dir().to_path_buf()
        } else {
            PathBuf::from(".").join(".data")
        }
    }

    pub fn get_config_dir() -> PathBuf {
        if let Some(s) = CONFIG_FOLDER.clone() {
            s
        } else if let Some(proj_dirs) = Self::project_directory() {
            proj_dirs.config_local_dir().to_path_buf()
        } else {
            PathBuf::from(".").join(".config")
        }
    }
    fn project_directory() -> Option<ProjectDirs> {
        ProjectDirs::from("ch", "skew", env!("CARGO_PKG_NAME"))
    }
    pub fn new(data_str: PathType, config_str: PathType) -> Self {
        let data = match data_str {
            PathType::Custom(p) => PathBuf::from(p),
            PathType::Default => Self::get_data_dir(),
            PathType::None => TempDir::new()
                .expect("Failed to create temporary data directory!")
                .into_path(),
        };
        let config = match config_str {
            PathType::Custom(p) => Some(PathBuf::from(p)),
            PathType::Default => Some(Self::get_config_dir()),
            PathType::None => None,
        };
        Self { data, config }
    }
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            data: Self::get_data_dir(),
            config: None,
        }
    }
}

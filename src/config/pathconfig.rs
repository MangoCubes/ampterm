use std::{env, path::PathBuf};

use directories::ProjectDirs;
use lazy_static::lazy_static;
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
    pub config: PathBuf,
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
    pub fn new(data_str: Option<String>, config_str: Option<String>) -> Self {
        let data = if let Some(p) = data_str {
            PathBuf::from(p)
        } else {
            Self::get_data_dir()
        };
        let config = if let Some(p) = config_str {
            PathBuf::from(p)
        } else {
            Self::get_config_dir()
        };
        Self { data, config }
    }
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            data: Self::get_data_dir(),
            config: Self::get_config_dir(),
        }
    }
}

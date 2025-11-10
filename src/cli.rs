use clap::Parser;

use crate::config::Config;

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli {
    /// Tick rate, i.e. number of ticks per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 4.0)]
    pub tick_rate: f64,

    /// Frame rate, i.e. number of frames per second
    #[arg(short, long, value_name = "FLOAT", default_value_t = 60.0)]
    pub frame_rate: f64,

    /// Specifies the *directory* of the config to load. This directory is expected to contain
    /// files like "config.json".
    #[arg(short, long)]
    pub config: Option<String>,

    /// Specifies the *directory* of the data store. This directory is expected to contain the
    /// database file.
    #[arg(short, long)]
    pub data: Option<String>,

    /// Do not use any config other than the preset. Incompatible with --config.
    #[arg(long, default_value_t = false)]
    pub no_config: bool,

    /// Do not save any data to the database. Incompatible with --data.
    #[arg(long, default_value_t = false)]
    pub no_data: bool,
}

impl Cli {
    pub fn is_valid(&self) -> Option<String> {
        if matches!(self.config, Some(_)) && self.no_config {
            return Some("Incompatible flags set: --config and --no-config".to_string());
        };
        if matches!(self.data, Some(_)) && self.no_data {
            return Some("Incompatible flags set: --data and --no-data".to_string());
        };
        None
    }
}

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "-",
    env!("VERGEN_GIT_DESCRIBE"),
    " (",
    env!("VERGEN_BUILD_DATE"),
    ")"
);

pub fn version() -> String {
    let author = clap::crate_authors!();

    // let current_exe_path = PathBuf::from(clap::crate_name!()).display().to_string();
    let config_dir_path = Config::get_config_dir().display().to_string();
    let data_dir_path = Config::get_data_dir().display().to_string();

    format!(
        "\
{VERSION_MESSAGE}

Authors: {author}

Config directory: {config_dir_path}
Data directory: {data_dir_path}"
    )
}

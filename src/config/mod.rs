use std::sync::Arc;

use args::Args;
use clap::Parser;
use parser_config::ParserConfig;

pub mod args;
mod colors;
pub mod log_options;
mod meta;
pub mod options;
pub mod parser_config;
pub mod state;
pub mod weidu_log_options;

pub const CARGO_PKG_NAME: &str = "mod_installer";

pub const LONG: &str = r"

  /\/\   ___   __| | (_)_ __  ___| |_ __ _| | | ___ _ __
 /    \ / _ \ / _` | | | '_ \/ __| __/ _` | | |/ _ \ '__|
/ /\/\ \ (_) | (_| | | | | | \__ \ || (_| | | |  __/ |
\/    \/\___/ \__,_| |_|_| |_|___/\__\__,_|_|_|\___|_|
";

#[cfg(target_os = "windows")]
const WEIDU_DL: &str =
  "https://github.com/WeiDUorg/weidu/releases/download/v249.00/WeiDU-Windows-249-amd64.zip";
#[cfg(target_os = "windows")]
const WEIDU_FOLDER_PATH: &str = "WeiDU-Windows";
#[cfg(target_os = "windows")]
pub const WEIDU_FILE_NAME: &str = "weidu.exe";

#[cfg(target_os = "macos")]
const WEIDU_DL: &str =
  "https://github.com/WeiDUorg/weidu/releases/download/v249.00/WeiDU-Mac-249.zip";
#[cfg(target_os = "macos")]
const WEIDU_FOLDER_PATH: &str = "WeiDU-Mac";
#[cfg(target_os = "macos")]
pub const WEIDU_FILE_NAME: &str = "weidu";

#[cfg(target_os = "linux")]
const WEIDU_DL: &str =
  "https://github.com/WeiDUorg/weidu/releases/download/v249.00/WeiDU-Linux-249-amd64.zip";
#[cfg(target_os = "linux")]
const WEIDU_FOLDER_PATH: &str = "WeiDU-Linux";
#[cfg(target_os = "linux")]
pub const WEIDU_FILE_NAME: &str = "weidu";

pub struct Config {
  pub args: Args,
  pub parser: Arc<ParserConfig>,
}

impl Config {
  pub fn new(parser_config_location: &str) -> Self {
    let parser_config: Arc<ParserConfig> = if let Ok(config) =
      confy::load::<ParserConfig>(CARGO_PKG_NAME, parser_config_location)
      && config.metadata.mod_installer_version == env!("CARGO_PKG_VERSION")
    {
      log::debug!("Using existing config: {:?}", config);
      Arc::new(config)
    } else {
      log::debug!("Creating new config");
      let config = Arc::new(ParserConfig::default());
      let _ = confy::store(
        CARGO_PKG_NAME,
        parser_config_location,
        config.clone().as_ref(),
      );
      config
    };
    Self {
      args: Args::parse(),
      parser: parser_config,
    }
  }
}

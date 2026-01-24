use std::sync::Arc;

use args::{Args, CARGO_PKG_NAME};
use clap::Parser;
use parser_config::ParserConfig;

pub mod args;
mod colors;
mod meta;
pub mod parser_config;
pub mod state;

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
            Arc::new(config)
        } else {
            Arc::new(ParserConfig::default())
        };
        Self {
            args: Args::parse(),
            parser: parser_config,
        }
    }
}

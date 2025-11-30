use std::sync::Arc;

use args::{Args, CARGO_PKG_NAME};
use clap::Parser;
use parser_config::ParserConfig;

use crate::PARSER_CONFIG_LOCATION;

pub(crate) mod args;
mod colors;
mod meta;
pub(crate) mod parser_config;

pub(crate) struct Config {
    pub(crate) args: Args,
    pub(crate) parser: Arc<ParserConfig>,
}

impl Config {
    pub(crate) fn new() -> Self {
        let parser_config: Arc<ParserConfig> = if let Ok(config) =
            confy::load::<ParserConfig>(CARGO_PKG_NAME, PARSER_CONFIG_LOCATION)
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

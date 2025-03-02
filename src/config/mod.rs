use std::sync::Arc;

use args::{Args, CARGO_PKG_NAME};
use clap::Parser;
use parser_config::ParserConfig;

use crate::PARSER_CONFIG_LOCATION;

pub(crate) mod args;
mod colors;
pub(crate) mod parser_config;

pub(crate) struct Config {
    pub(crate) args: Args,
    pub(crate) parser: Arc<ParserConfig>,
}

impl Config {
    pub(crate) fn new() -> Config {
        let parser_config: Arc<ParserConfig> =
            if let Ok(config) = confy::load(CARGO_PKG_NAME, PARSER_CONFIG_LOCATION) {
                Arc::new(config)
            } else {
                Arc::new(ParserConfig::default())
            };
        Config {
            args: Args::parse(),
            parser: parser_config,
        }
    }
}

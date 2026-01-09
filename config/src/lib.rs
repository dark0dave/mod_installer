use clap::Parser;
use std::sync::Arc;

pub mod args;
pub mod colors;
pub mod meta;
pub mod parser_config;
pub mod state;

use crate::{
    args::Args,
    parser_config::{LOCATION, ParserConfig},
};

pub struct Config {
    pub args: Args,
    pub parser: Arc<ParserConfig>,
}

impl Config {
    pub fn new(app_name: &str) -> Self {
        let parser_config: Arc<ParserConfig> =
            if let Ok(config) = confy::load::<ParserConfig>(app_name, LOCATION) {
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

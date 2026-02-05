use std::process::ExitCode;

use config::{
    Config,
    args::{CommandType, Scan},
};
use env_logger::Env;
use installers::{eet_install, normal_install};
use scan::scan_components;
use scan_langauges::scan_langauges;

mod component;
mod installers;
mod log_file;
mod raw_reciever;
mod scan;
mod scan_langauges;
mod utils;
mod weidu;
mod weidu_parser;

const PARSER_CONFIG_LOCATION: &str = "parser";

fn main() -> ExitCode {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = Config::new(PARSER_CONFIG_LOCATION);

    log::debug!("{:?}", config.args);

    let status = match config.args.command {
        CommandType::Normal(command) => normal_install(&command, config.parser.clone()),
        CommandType::Eet(command) => eet_install(&command, config.parser.clone()),
        CommandType::Scan(Scan::Langauges(command)) => scan_langauges(&command),
        CommandType::Scan(Scan::Components(command)) => scan_components(&command),
    };

    match status {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{err}");
            ExitCode::FAILURE
        }
    }
}

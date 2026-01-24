use std::process::ExitCode;

use config::{Config, args::CommandType};
use env_logger::Env;
use installers::{eet_install, normal_install};
use scan::scan;

mod component;
mod installers;
mod log_file;
mod scan;
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
        CommandType::Scan(command) => scan(&command),
    };

    match status {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{err}");
            ExitCode::FAILURE
        }
    }
}

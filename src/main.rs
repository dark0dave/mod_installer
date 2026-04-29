use std::process::ExitCode;

use config::{
    Config,
    args::{CommandType, Scan},
};
use env_logger::Env;
use installers::{eet_install, normal_install};
use scan_components::scan_components;
use scan_langauges::scan_langauges;

mod installers;
mod internal_log;
mod parser;
mod raw_reciever;
mod runner;
mod scan_components;
mod scan_langauges;
mod utils;
mod weidu_batched_components;
mod weidu_batched_install_order;
mod weidu_component;
mod weidu_install_block;

const PARSER_CONFIG_LOCATION: &str = "parser";

fn main() -> ExitCode {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = Config::new(PARSER_CONFIG_LOCATION);

    log::debug!("{:?}", config.args);

    let status = match config.args.command {
        CommandType::Normal(command) => normal_install(&command, config.parser.clone()),
        CommandType::Eet(command) => eet_install(&command, config.parser.clone()),
        CommandType::Scan(Scan::Languages(command)) => scan_langauges(&command),
        CommandType::Scan(Scan::Components(command)) => scan_components(&command),
    };

    match status {
        Err(err) => {
            log::error!("{err}");
            ExitCode::FAILURE
        }
        _ => ExitCode::SUCCESS,
    }
}

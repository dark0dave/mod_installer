use std::process::ExitCode;

use config::{
    Config,
    args::{CommandType, Scan},
};
use env_logger::Env;
use installers::{eet_install, normal_install};
use scan::components::scan_components;
use scan::languages::scan_langauges;

use utils::find_all_mods;

mod config;
mod installers;
mod internal_log;
mod parser;
mod raw_reciever;
mod runner;
mod scan;
mod utils;
mod weidu;

const PARSER_CONFIG_LOCATION: &str = "parser";

fn main() -> ExitCode {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = Config::new(PARSER_CONFIG_LOCATION);

    log::debug!("{:?}", config.args);

    let status = match config.args.command {
        CommandType::Normal(command) => normal_install(
            &command,
            config.parser.clone(),
            &mut find_all_mods(&command.options.mod_directories, command.options.depth),
        ),
        CommandType::Eet(command) => eet_install(
            &command,
            config.parser.clone(),
            &mut find_all_mods(&command.options.mod_directories, command.options.depth),
        ),
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

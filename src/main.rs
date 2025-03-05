use std::process::ExitCode;

use config::parser_config::PARSER_CONFIG_LOCATION;
use config::{args::InstallType, Config};
use env_logger::Env;
use installers::{eet_install, normal_install};

mod component;
mod config;
mod installers;
mod log_file;
mod state;
mod utils;
mod weidu;
mod weidu_parser;

fn main() -> ExitCode {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = Config::new();

    let status = match config.args.command {
        InstallType::Normal(command) => normal_install(
            &command.log_file,
            &command.game_directory,
            &command.new_game_directory,
            &command.options,
            config.parser.clone(),
        ),
        InstallType::Eet(command) => eet_install(&command, config.parser.clone()),
    };

    match status {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{}", err);
            ExitCode::FAILURE
        }
    }
}

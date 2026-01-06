use std::process::ExitCode;

use config::{Config, args::InstallType};
use env_logger::Env;
use installers::{eet_install, normal_install};

mod component;
mod installers;
mod log_file;
mod utils;
mod weidu;
mod weidu_parser;

const NAME: &str = env!("CARGO_PKG_NAME");

fn main() -> ExitCode {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = Config::new(NAME);

    log::debug!("{:?}", config.args);

    let status = match config.args.command {
        InstallType::Normal(command) => normal_install(&command, config.parser.clone()),
        InstallType::Eet(command) => eet_install(&command, config.parser.clone()),
    };

    match status {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{err}");
            ExitCode::FAILURE
        }
    }
}

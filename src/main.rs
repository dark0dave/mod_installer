use std::{collections::HashMap, process::ExitCode, sync::Arc};

use args::{Args, CARGO_PKG_NAME};
use clap::Parser;
use env_logger::Env;
use parser_config::ParserConfig;
use utils::find_mods;

use crate::{
    utils::{copy_mod_folder, mod_folder_present_in_game_directory, search_mod_folders},
    weidu::{install, InstallationResult},
};

mod args;
mod component;
mod log_file;
mod parser_config;
mod state;
mod utils;
mod weidu;
mod weidu_parser;

fn main() -> ExitCode {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    log::info!(
        r"

                 /\/\   ___   __| | (_)_ __  ___| |_ __ _| | | ___ _ __
                /    \ / _ \ / _` | | | '_ \/ __| __/ _` | | |/ _ \ '__|
               / /\/\ \ (_) | (_| | | | | | \__ \ || (_| | | |  __/ |
               \/    \/\___/ \__,_| |_|_| |_|___/\__\__,_|_|_|\___|_|
        "
    );
    let args = Args::parse();
    let parser_config: Arc<ParserConfig> = match confy::load(CARGO_PKG_NAME, "config") {
        Ok(config) => Arc::new(config),
        Err(err) => {
            log::error!("Internal error with config crate, {:?}", err);
            return ExitCode::FAILURE;
        }
    };

    let mods_to_be_installed = match find_mods(
        args.log_file,
        args.skip_installed,
        args.game_directory.clone(),
        args.strict_matching,
    ) {
        Ok(mods) => mods,
        Err(err) => {
            log::error!("Failed to find log file, {:?}", err);
            return ExitCode::FAILURE;
        }
    };

    let mut mod_folder_cache = HashMap::new();
    for weidu_mod in &mods_to_be_installed {
        let mod_folder = mod_folder_cache
            .entry(weidu_mod.tp_file.clone())
            .or_insert_with(|| search_mod_folders(&args.mod_directories, weidu_mod, args.depth));

        log::debug!("Found mod folder {:?}, for mod {:?}", mod_folder, weidu_mod);

        if !mod_folder_present_in_game_directory(&args.game_directory, &weidu_mod.name) {
            log::debug!(
                "Copying mod directory, from {:?} to, {:?}",
                mod_folder,
                args.game_directory.join(&weidu_mod.name)
            );
            copy_mod_folder(&args.game_directory, mod_folder)
        }
        log::info!("Installing mod {:?}", &weidu_mod);
        match install(
            &args.weidu_binary,
            &args.game_directory,
            parser_config.clone(),
            weidu_mod,
            &args.language,
            &args.weidu_log_mode,
            args.timeout,
        ) {
            InstallationResult::Fail(message) => {
                log::error!(
                    "Failed to install mod {}, error is '{}'",
                    weidu_mod.name,
                    message
                );
                return ExitCode::FAILURE;
            }
            InstallationResult::Success => {
                log::info!("Installed mod {:?}", &weidu_mod);
            }
            InstallationResult::Warnings => {
                if args.abort_on_warnings {
                    log::error!("Installed mod {:?} with warnings, stopping", weidu_mod);
                    break;
                } else {
                    log::warn!("Installed mod {:?} with warnings, keep going", weidu_mod);
                }
            }
        }
    }
    ExitCode::SUCCESS
}

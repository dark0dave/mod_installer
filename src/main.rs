use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::ExitCode,
    sync::Arc,
};

use args::{Args, Eet, InstallType, Options, CARGO_PKG_NAME};
use clap::Parser;
use env_logger::Env;
use parser_config::ParserConfig;
use utils::find_mods;

use crate::{
    utils::{copy_mod_folder, mod_folder_present_in_game_directory, search_mod_folders},
    weidu::{install, InstallationResult},
};

mod args;
mod colors;
mod component;
mod log_file;
mod parser_config;
mod state;
mod utils;
mod weidu;
mod weidu_parser;

fn normal_install(
    log_file: &Path,
    game_directory: &PathBuf,
    options: &Options,
    parser_config: Arc<ParserConfig>,
) -> ExitCode {
    let mods_to_be_installed = match find_mods(
        log_file,
        options.skip_installed,
        game_directory,
        options.strict_matching,
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
            .or_insert_with(|| {
                search_mod_folders(&options.mod_directories, weidu_mod, options.depth)
            });

        log::debug!("Found mod folder {:?}, for mod {:?}", mod_folder, weidu_mod);

        if !mod_folder_present_in_game_directory(game_directory, &weidu_mod.name) {
            log::debug!(
                "Copying mod directory, from {:?} to, {:?}",
                mod_folder,
                game_directory.join(&weidu_mod.name)
            );
            copy_mod_folder(game_directory, mod_folder)
        }
        log::info!("Installing mod {:?}", &weidu_mod);
        match install(
            &options.weidu_binary,
            game_directory,
            parser_config.clone(),
            weidu_mod,
            &options.language,
            &options.weidu_log_mode,
            options.timeout,
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
                if options.abort_on_warnings {
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

fn eet_install(command: &Eet, parser_config: Arc<ParserConfig>) -> ExitCode {
    // Handle BG1EE Install
    let exit_code = normal_install(
        &command.bg1_log_file,
        &command.bg1_game_directory,
        &command.options,
        parser_config.clone(),
    );

    if exit_code != ExitCode::SUCCESS {
        return exit_code;
    }

    // Handle BG2EE Install
    normal_install(
        &command.bg2_log_file,
        &command.bg2_game_directory,
        &command.options,
        parser_config.clone(),
    )
}

fn main() -> ExitCode {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    let parser_config: Arc<ParserConfig> = match confy::load(CARGO_PKG_NAME, "config") {
        Ok(config) => Arc::new(config),
        Err(err) => {
            log::error!("Internal error with config crate, {:?}", err);
            return ExitCode::FAILURE;
        }
    };

    match args.command {
        InstallType::Normal(command) => normal_install(
            &command.log_file,
            &command.game_directory,
            &command.options,
            parser_config.clone(),
        ),
        InstallType::Eet(command) => eet_install(&command, parser_config.clone()),
    }
}

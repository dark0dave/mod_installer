use std::{collections::HashMap, error::Error, path::Path, process::ExitCode, sync::Arc};

use config::parser_config::{ParserConfig, PARSER_CONFIG_LOCATION};
use config::{
    args::{Eet, InstallType, Options},
    Config,
};
use env_logger::Env;
use utils::{clone_directory, find_mods};

use crate::{
    utils::{copy_folder, mod_folder_present_in_game_directory, search_mod_folders},
    weidu::{install, InstallationResult},
};

mod component;
mod config;
mod log_file;
mod state;
mod utils;
mod weidu;
mod weidu_parser;

fn normal_install(
    log_file: &Path,
    game_directory: &Path,
    options: &Options,
    parser_config: Arc<ParserConfig>,
) -> Result<(), Box<dyn Error>> {
    let mods_to_be_installed = match find_mods(
        log_file,
        options.skip_installed,
        game_directory,
        options.strict_matching,
    ) {
        Ok(mods) => mods,
        Err(err) => {
            return Err(format!("Failed to find weidu log file, {:?}", err).into());
        }
    };

    let mut mod_folder_cache = HashMap::new();
    for weidu_mod in &mods_to_be_installed {
        let mod_folder = mod_folder_cache
            .entry(weidu_mod.tp_file.clone())
            .or_insert_with(|| {
                search_mod_folders(&options.mod_directories, weidu_mod, options.depth).unwrap()
            });

        log::debug!("Found mod folder {:?}, for mod {:?}", mod_folder, weidu_mod);

        if !mod_folder_present_in_game_directory(game_directory, &weidu_mod.name) {
            log::info!(
                "Copying mod directory, from {:?} to, {:?}",
                mod_folder,
                game_directory.join(&weidu_mod.name)
            );
            copy_folder(mod_folder, game_directory.join(&weidu_mod.name))?;
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
                return Err(format!(
                    "Failed to install mod {}, error is '{}'",
                    weidu_mod.name, message
                )
                .into());
            }
            InstallationResult::Success => {
                log::info!("Installed mod {:?}", &weidu_mod);
            }
            InstallationResult::Warnings => {
                if options.abort_on_warnings {
                    return Err(
                        format!("Installed mod {:?} with warnings, stopping", weidu_mod).into(),
                    );
                } else {
                    log::warn!("Installed mod {:?} with warnings, keep going", weidu_mod);
                }
            }
        }
    }
    Ok(())
}

fn eet_install(command: &Eet, parser_config: Arc<ParserConfig>) -> Result<(), Box<dyn Error>> {
    log::info!("Beginning pre eet install process");
    let game_directory = if command.create_directories {
        clone_directory(
            &command.bg1_game_directory,
            &command.create_directories_prefix,
            "pre-eet",
        )?
    } else {
        command.bg1_game_directory.clone()
    };
    normal_install(
        &command.bg1_log_file,
        &game_directory,
        &command.options,
        parser_config.clone(),
    )?;

    log::info!("Beginning eet install process");
    let game_directory = if command.create_directories {
        clone_directory(
            &command.bg2_game_directory,
            &command.create_directories_prefix,
            "eet",
        )?
    } else {
        command.bg2_game_directory.clone()
    };
    normal_install(
        &command.bg2_log_file,
        &game_directory,
        &command.options,
        parser_config.clone(),
    )
}

fn main() -> ExitCode {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = Config::new();

    let status = match config.args.command {
        InstallType::Normal(command) => normal_install(
            &command.log_file,
            &command.game_directory,
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

use std::path::PathBuf;
use std::{collections::HashMap, error::Error, path::Path, sync::Arc};

use config::args::{Eet, Normal, Options};
use config::parser_config::ParserConfig;

use crate::utils::{copy_folder, find_mods, mod_folder_present_in_game_directory};
use crate::utils::{delete_folder, get_last_installed, search_or_download};
use crate::weidu::{self, WeiduExitStatus};

pub(crate) fn normal_install(
    command: &Normal,
    parser_config: Arc<ParserConfig>,
) -> Result<(), Box<dyn Error>> {
    log::info!("Beginning normal install process");
    let game_directory = if let Some(new_directory) = &command.generate_directory {
        copy_folder(
            &command.game_directory,
            new_directory,
            command.options.casefold,
        )?;
        new_directory.clone()
    } else {
        command.game_directory.clone()
    };

    install(
        &command.log_file,
        &game_directory,
        &command.options,
        parser_config.clone(),
        None,
    )
}

pub(crate) fn eet_install(
    command: &Eet,
    parser_config: Arc<ParserConfig>,
) -> Result<(), Box<dyn Error>> {
    log::info!("Beginning pre eet install process");
    let pre_eet_game_directory = if let Some(new_directory) = &command.new_pre_eet_dir {
        copy_folder(
            &command.bg1_game_directory,
            new_directory,
            command.options.casefold,
        )?;
        new_directory.clone()
    } else {
        command.bg1_game_directory.clone()
    };

    install(
        &command.bg1_log_file,
        &pre_eet_game_directory,
        &command.options,
        parser_config.clone(),
        None,
    )?;

    log::info!("Beginning eet install process");
    let game_directory = if let Some(new_directory) = &command.new_eet_dir {
        copy_folder(
            &command.bg2_game_directory,
            new_directory,
            command.options.casefold,
        )?;
        new_directory.clone()
    } else {
        command.bg2_game_directory.clone()
    };
    install(
        &command.bg2_log_file,
        &game_directory,
        &command.options,
        parser_config.clone(),
        Some(&pre_eet_game_directory),
    )
}

fn install(
    log_file: &Path,
    game_directory: &Path,
    options: &Options,
    parser_config: Arc<ParserConfig>,
    pre_eet_game_directory: Option<&PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let mods_to_be_installed = match find_mods(
        log_file,
        options.skip_installed,
        game_directory,
        options.strict_matching,
    ) {
        Ok(mods) => mods,
        Err(err) => {
            return Err(format!("Failed to find weidu log file, {err:?}").into());
        }
    };

    let mut mod_folder_cache = HashMap::new();
    for weidu_mod in &mods_to_be_installed {
        let mod_folder = mod_folder_cache
            .entry(weidu_mod.tp_file.clone())
            .or_insert_with(|| {
                search_or_download(options, weidu_mod).expect("Couldn't find mod exiting")
            });

        log::debug!("Found mod folder {mod_folder:?}, for mod {weidu_mod:?}");

        if options.overwrite {
            delete_folder(game_directory.join(&weidu_mod.name))?;
        }

        if !mod_folder_present_in_game_directory(game_directory, &weidu_mod.name) {
            log::debug!(
                "Copying mod directory, from {:?} to, {:?}",
                mod_folder,
                game_directory.join(&weidu_mod.name)
            );
            copy_folder(mod_folder, game_directory.join(&weidu_mod.name), false)?;
        }
        log::info!("Installing mod {:?}", &weidu_mod);
        let bg1_game_direcotry = if weidu_mod
            .component_name
            .to_lowercase()
            .eq("eet core (resource importation)")
        {
            pre_eet_game_directory
        } else {
            None
        };
        match weidu::install(
            game_directory,
            parser_config.clone(),
            weidu_mod,
            options,
            bg1_game_direcotry,
        )? {
            WeiduExitStatus::Success => {
                let last_installed = get_last_installed(game_directory)?;
                if options.check_last_installed && last_installed.ne(weidu_mod) {
                    return Err(format!(
                        "Last installed {last_installed:?} does not match component installed: {weidu_mod:?}"
                    )
                    .into());
                }
                log::info!("Installed mod {:?}", &last_installed);
            }
            WeiduExitStatus::Warnings(msg) if options.abort_on_warnings => {
                return Err(format!(
                    "Installed mod {weidu_mod:?} with warnings: \n{msg}\n, stopping"
                )
                .into());
            }
            WeiduExitStatus::Warnings(msg) => {
                log::warn!("Installed mod {weidu_mod:?} with warnings:  \n{msg}\n")
            }
        }
    }
    Ok(())
}

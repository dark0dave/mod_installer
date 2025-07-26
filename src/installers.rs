use std::path::PathBuf;
use std::{collections::HashMap, error::Error, path::Path, sync::Arc};

use crate::utils::{delete_folder, get_last_installed, search_or_download};
use crate::{
    config::args::{Eet, Options},
    config::parser_config::ParserConfig,
    utils::{
        clone_directory, copy_folder, find_mods, find_parent_folder,
        mod_folder_present_in_game_directory,
    },
    weidu::{install, InstallationResult},
};

const EET: &str = "eet";
const PRE_EET: &str = "pre-eet";

pub(crate) fn normal_install(
    log_file: &Path,
    game_dir: &Path,
    new_game_directory: &Option<PathBuf>,
    options: &Options,
    parser_config: Arc<ParserConfig>,
) -> Result<(), Box<dyn Error>> {
    let game_directory = if let Some(new_game_dir) = new_game_directory.clone() {
        clone_directory(game_dir, &new_game_dir)?
    } else {
        game_dir.to_path_buf()
    };

    let mods_to_be_installed = match find_mods(
        log_file,
        options.skip_installed,
        &game_directory,
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
            let possible_mod_directory = game_directory.join(&weidu_mod.name);
            delete_folder(&possible_mod_directory)?;
        }

        if !mod_folder_present_in_game_directory(&game_directory, &weidu_mod.name) {
            log::debug!(
                "Copying mod directory, from {:?} to, {:?}",
                mod_folder,
                game_directory.join(&weidu_mod.name)
            );
            copy_folder(mod_folder, game_directory.join(&weidu_mod.name))?;
        }
        log::info!("Installing mod {:?}", &weidu_mod);
        match install(&game_directory, parser_config.clone(), weidu_mod, options) {
            InstallationResult::Fail(message) => {
                return Err(format!(
                    "Failed to install mod {}, error is '{}'",
                    weidu_mod.name, message
                )
                .into());
            }
            InstallationResult::Success => {
                let last_installed = get_last_installed(game_dir)?;
                if options.check_last_installed && last_installed.ne(weidu_mod) {
                    return Err(format!(
                        "Last installed {last_installed:?} does not match component installed: {weidu_mod:?}"
                    )
                    .into());
                }
                log::info!("Installed mod {:?}", &last_installed);
            }
            InstallationResult::Warnings(msg) => {
                log::warn!("{msg}");
                if options.abort_on_warnings {
                    return Err(
                        format!("Installed mod {weidu_mod:?} with warnings, stopping").into(),
                    );
                } else {
                    log::warn!("Installed mod {weidu_mod:?} with warnings, keep going");
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn eet_install(
    command: &Eet,
    parser_config: Arc<ParserConfig>,
) -> Result<(), Box<dyn Error>> {
    log::info!("Beginning pre eet install process");
    let new_game_directory: Option<PathBuf> = if command.generate_directories {
        if command.new_pre_eet_dir.is_none() {
            Some(find_parent_folder(&command.bg1_game_directory)?.join(PRE_EET))
        } else {
            command.new_pre_eet_dir.clone()
        }
    } else {
        None
    };
    normal_install(
        &command.bg1_log_file,
        &command.bg1_game_directory,
        &new_game_directory,
        &command.options,
        parser_config.clone(),
    )?;

    log::info!("Beginning eet install process");
    let new_game_directory: Option<PathBuf> = if command.generate_directories {
        if command.new_eet_dir.is_none() {
            Some(find_parent_folder(&command.bg2_game_directory)?.join(EET))
        } else {
            command.new_eet_dir.clone()
        }
    } else {
        None
    };
    normal_install(
        &command.bg2_log_file,
        &command.bg2_game_directory,
        &new_game_directory,
        &command.options,
        parser_config.clone(),
    )
}

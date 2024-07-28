use std::collections::HashMap;

use args::Args;
use clap::Parser;
use env_logger::Env;
use log_file::LogFile;

use crate::{
    utils::{copy_mod_folder, mod_folder_present_in_game_directory, search_mod_folders},
    weidu::{install, InstallationResult},
};

mod args;
mod component;
mod log_file;
mod state;
mod utils;
mod weidu;
mod weidu_parser;

fn main() {
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

    let mut mods = LogFile::try_from(args.log_file).expect("Could not open log file");
    let number_of_mods_found = mods.len();
    let mods_to_be_installed = if args.skip_installed {
        let existing_weidu_log_file_path = args.game_directory.join("weidu").with_extension("log");
        if let Ok(installed_mods) = LogFile::try_from(existing_weidu_log_file_path) {
            for installed_mod in &installed_mods {
                mods.retain(|mod_to_install| installed_mod != mod_to_install);
            }
        }
        mods
    } else {
        mods
    };

    log::info!(
        "Number of mods found: {}, Number of mods to be installed: {}",
        number_of_mods_found,
        mods_to_be_installed.len()
    );

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
            weidu_mod,
            &args.language,
            &args.weidu_log_mode,
            args.timeout,
        ) {
            InstallationResult::Fail(message) => {
                panic!(
                    "Failed to install mod {}, error is '{}'",
                    weidu_mod.name, message
                );
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
}

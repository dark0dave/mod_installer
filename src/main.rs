use std::collections::HashMap;

use args::Args;
use clap::Parser;
use env_logger::Env;

use crate::{
    mod_component::parse_weidu_log,
    utils::{
        copy_mod_folder, create_weidu_log_if_not_exists, mod_folder_present_in_game_directory,
        search_mod_folders,
    },
    weidu::{install, InstallationResult},
};

mod args;
mod mod_component;
mod utils;
mod weidu;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    println!(
        r"
                 /\/\   ___   __| | (_)_ __  ___| |_ __ _| | | ___ _ __
                /    \ / _ \ / _` | | | '_ \/ __| __/ _` | | |/ _ \ '__|
               / /\/\ \ (_) | (_| | | | | | \__ \ || (_| | | |  __/ |
               \/    \/\___/ \__,_| |_|_| |_|___/\__\__,_|_|_|\___|_|
        "
    );
    let args = Args::parse();

    let installed_log_path = create_weidu_log_if_not_exists(&args.game_directory);

    let mods = parse_weidu_log(args.log_file);
    let number_of_mods_found = mods.len();
    let mods_to_be_installed = if args.skip_installed {
        let installed_mods = parse_weidu_log(installed_log_path);
        mods.iter()
            .filter_map(|weidu_mod| {
                if !installed_mods.contains(weidu_mod) {
                    Some(weidu_mod.clone())
                } else {
                    None
                }
            })
            .collect()
    } else {
        mods
    };

    log::debug!(
        "Number of mods found: {}, Number of mods to be installed: {}",
        number_of_mods_found,
        mods_to_be_installed.len()
    );

    let mut mod_folder_cache = HashMap::new();
    for weidu_mod in mods_to_be_installed {
        let mod_folder = mod_folder_cache
            .entry(weidu_mod.tp_file.clone())
            .or_insert_with(|| {
                search_mod_folders(&args.mod_directories, &weidu_mod.clone(), args.depth)
            });

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
        match  install(
            &args.weidu_binary,
            &args.game_directory,
            &weidu_mod,
            &args.language,
        ) {
            InstallationResult::Fail(message) => {
                panic!("Failed to install mod {}, error is '{}'", weidu_mod.name, message);
            }
            InstallationResult::Success => {
                log::info!("Installed mod {:?}", &weidu_mod);
            }
            InstallationResult::Warnings => {
                if args.stop_on_warnings {
                    log::info!("Installed mod {:?} with warnings, stopping", &weidu_mod);
                    break;
                } else {
                    log::info!("Installed mod {:?} with warnings, keep going", &weidu_mod);
                }
            }
        }
    }
}

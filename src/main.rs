use args::Args;
use clap::Parser;
use env_logger::Env;

use crate::{
    mod_component::parse_weidu_log,
    utils::{
        copy_mod_folder, create_weidu_log_if_not_exists, find_mod_folder,
        mod_folder_present_in_game_directory,
    },
    weidu::{generate_args, install},
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

    create_weidu_log_if_not_exists(&args.game_directory);

    for weidu_mod in parse_weidu_log(args.log_file) {
        let mod_folder_locations = args
            .mod_directories
            .iter()
            .find_map(|mod_folder| find_mod_folder(&weidu_mod, mod_folder));

        let mod_folder = if let Some(mod_folder) = mod_folder_locations {
            mod_folder
        } else {
            log::error!("Could not find {:#?} mod folder ", weidu_mod);
            panic!();
        };
        log::info!("Found mod folder {:?}, for mod {:?}", mod_folder, weidu_mod);

        if !mod_folder_present_in_game_directory(&args.game_directory, &weidu_mod.name) {
            log::info!(
                "Copying mod directory, from {:?} to, {:?}",
                mod_folder,
                args.game_directory.clone().join(weidu_mod.name.clone())
            );
            copy_mod_folder(&args.game_directory, &mod_folder)
        }
        let weidu_args = generate_args(&weidu_mod, &args.language);
        install(&args.weidu_binary, &args.game_directory, weidu_args);
    }
}

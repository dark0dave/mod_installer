use std::{path::PathBuf, process::Command};

use crate::mod_component::ModComponent;

pub fn generate_args(weidu_mod: &ModComponent, language: &str) -> Vec<String> {
    format!("{mod_install_path} --yes --ask-only {component} --use-lang {game_lang} --language {mod_lang}", mod_install_path = weidu_mod.install_path, component = weidu_mod.component, mod_lang = weidu_mod.lang, game_lang = language).split(' ').map(|x|x.to_string()).collect()
}

pub fn install(weidu_binary: &PathBuf, game_directory: &PathBuf, weidu_args: Vec<String>) {
    log::debug!("{:#?}", weidu_args);
    let weidu_process = Command::new(weidu_binary)
        .current_dir(game_directory)
        .args(weidu_args)
        .spawn()
        .expect("Weidu failed to start");

    match weidu_process.wait_with_output() {
        Ok(output) if !output.status.success() => {
            log::error!("Failed to install mod with error: {:?}", output.stderr);
            panic!();
        }
        Err(error) => {
            log::error!("Command did not gracefully terminate, {}", error);
            panic!();
        }
        _ => {}
    }
}

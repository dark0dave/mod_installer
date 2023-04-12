use std::{path::PathBuf, process::Command};

use crate::mod_component::ModComponent;

pub fn generate_args(weidu_mod: &ModComponent, language: &str) -> Vec<String> {
    format!("{mod_install_path} --quick-log --force-install {component} --use-lang {game_lang} --language {mod_lang}", mod_install_path = weidu_mod.install_path, component = weidu_mod.component, mod_lang = weidu_mod.lang, game_lang = language).split(' ').map(|x|x.to_string()).collect()
}

pub fn install(weidu_binary: &PathBuf, game_directory: &PathBuf, weidu_args: Vec<String>) {
    log::trace!("{:#?}", weidu_args);
    let mut command = Command::new(weidu_binary);
    let weidu_process = command.current_dir(game_directory).args(weidu_args);

    match weidu_process.output() {
        Ok(output) if !output.status.success() => {
            let lines = std::str::from_utf8(&output.stdout)
                .unwrap_or_default()
                .split('\n');
            if lines
                .clone()
                .any(|x| x.starts_with("INSTALLED WITH WARNINGS"))
            {
                lines
                    .filter(|x| x.contains("WARNING:"))
                    .for_each(|warning| log::warn!("{:#?}", warning));
            } else {
                for line in lines {
                    log::error!("{}", line);
                }
                panic!("Failed to install mod");
            }
        }
        Err(error) => {
            log::error!("Command did not gracefully terminate, {}", error);
            panic!();
        }
        _ => {}
    }
}

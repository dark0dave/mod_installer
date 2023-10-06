use std::{
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::mod_component::ModComponent;

pub fn get_user_input() -> String {
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap_or_default();
    log::debug!("User input: {}", input);

    input.to_string()
}

pub fn generate_args(weidu_mod: &ModComponent, language: &str) -> Vec<String> {
    format!("{mod_name}/{mod_tp_file} --quick-log --yes --ask-only {component} --use-lang {game_lang} --language {mod_lang}", mod_name = weidu_mod.name, mod_tp_file = weidu_mod.tp_file, component = weidu_mod.component, mod_lang = weidu_mod.lang, game_lang = language).split(' ').map(|x|x.to_string()).collect()
}

pub fn install(weidu_binary: &PathBuf, game_directory: &PathBuf, weidu_args: Vec<String>) {
    log::trace!("{:#?}", weidu_args);
    let mut command = Command::new(weidu_binary);
    let weidu_process = command.current_dir(game_directory).args(weidu_args.clone());

    let mut child = weidu_process
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn weidu process");

    let mut reader = BufReader::new(child.stdout.as_mut().unwrap());
    let stdin = &mut child.stdin.take().expect("Failed to open stdin");

    let mut choice_flag = false;
    let mut failure_flag = false;
    while child.stderr.is_none() {
        let mut text = String::new();
        if reader.read_line(&mut text).is_ok() {
            if !text.is_empty() && !failure_flag {
                log::trace!("{}", text);
            } else {
                log::error!("{}", text);
            }

            if text.to_ascii_lowercase().contains("failure") {
                failure_flag = true;
            }

            if text.contains("Stopping installation because of error.") {
                log::error!("Weidu process failed with args: {:?}", weidu_args);
                panic!();
            }

            match text {
                // Choice
                _ if choice_flag => {
                    if !text.chars().nth(1).unwrap_or_default().is_numeric() {
                        stdin
                            .write_all(get_user_input().as_bytes())
                            .expect("Failed to write to stdin");
                        break;
                    }
                }
                x if x.starts_with("SKIPPING: ") || x.starts_with("Already Asked About") => {
                    stdin
                        .write_all("\n".as_bytes())
                        .expect("Failed to write to stdin");
                    log::debug!("Skiping component");
                    break;
                }
                x if (x.trim_end().ends_with("[Q]uit or choose one:")
                    || x.trim_end().starts_with("Enter "))
                    && !x.to_ascii_lowercase().starts_with("[r]e-install") =>
                {
                    log::trace!("Choice found");
                    choice_flag = true;
                }

                // Success
                x if x.contains("SUCCESSFULLY INSTALLED")
                    || x.starts_with("INSTALLED WITH WARNINGS") =>
                {
                    break;
                }

                // Install
                x if x.starts_with("Install") => {
                    stdin
                        .write_all("\n".as_bytes())
                        .expect("Failed to write to stdin");
                }
                x if x.starts_with("[I]nstall") => {
                    stdin
                        .write_all("I\n".as_bytes())
                        .expect("Failed to write to stdin");
                    log::debug!("Installing");
                }
                x if x.to_ascii_lowercase().starts_with("[r]e-install") => {
                    stdin
                        .write_all("Q\n".as_bytes())
                        .expect("Could not quit out");
                    log::debug!("Continue as already installed");
                    break;
                }
                _ => {}
            }
        } else {
            break;
        }
    }

    match child.wait_with_output() {
        Ok(output) if !output.status.success() => {
            panic!("{:#?}", output);
        }
        Err(err) => {
            panic!("Did not close properly: {}", err);
        }
        Ok(output) => {
            log::trace!("{:#?}", output);
        }
    }
}

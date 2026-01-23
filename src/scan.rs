use std::collections::HashSet;
use std::io::{BufReader, Read};
use std::{error::Error, path::PathBuf, process::ChildStdout, process::Command, process::Stdio};

use walkdir::WalkDir;

use crate::config::args::Scan;
use crate::weidu::LINE_SEPERATOR;

fn generate_args_for_list_components_with_game_dir(
    mod_path: &str,
    lang: &str,
    game_dir: &str,
) -> Vec<String> {
    vec![
        "--game".to_string(),
        game_dir.to_string(),
        "--list-components".to_string(),
        mod_path.to_string(),
        lang.to_string(),
        "--no-exit-pause".to_string(),
    ]
}

fn generate_args_for_list_lang(mod_path: &str) -> Vec<String> {
    vec![
        "--nogame".to_string(),
        "--list-languages".to_string(),
        mod_path.to_string(),
        "--no-exit-pause".to_string(),
    ]
}

fn find_mods(mod_dir: Vec<PathBuf>, depth: usize) -> Vec<PathBuf> {
    mod_dir
        .iter()
        .flat_map(|mod_dir| {
            WalkDir::new(mod_dir)
                .follow_links(true)
                .max_depth(depth)
                .into_iter()
                .flat_map(|entry| match entry {
                    Ok(entry)
                        if entry
                            .file_name()
                            .to_str()
                            .unwrap_or_default()
                            .ends_with(".tp2") =>
                    {
                        Some(entry.path().to_path_buf())
                    }
                    _ => None,
                })
        })
        .collect()
}

fn shorten_weidu_component_path_string(weidu_component_path: &str) -> String {
    let mut parts = weidu_component_path.splitn(3, "~");
    let mut component_path_string = parts
        .nth(1)
        .unwrap_or_default()
        .split(LINE_SEPERATOR)
        .collect::<Vec<&str>>()
        .into_iter()
        .rev();
    let (tail, head) = (
        component_path_string.next().unwrap_or_default(),
        component_path_string.next().unwrap_or_default(),
    );
    format!(
        "~{}{}{}~{}",
        head,
        LINE_SEPERATOR,
        tail,
        parts.last().unwrap_or_default()
    )
}

pub(crate) fn scan(command: &Scan) -> Result<(), Box<dyn Error>> {
    let mods = find_mods(
        command.options.mod_directories.clone(),
        command.options.depth,
    );
    log::trace!("{:?}", mods);

    for weidu_mod in mods {
        let mut mod_langs: HashSet<&str> = HashSet::new();
        let weidu_args_langs = generate_args_for_list_lang(weidu_mod.to_str().unwrap_or_default());
        let mut run = Command::new(command.options.weidu_binary.clone());
        let mut output = run
            .args(weidu_args_langs)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Could not run");
        if output.try_wait().is_err() {
            log::warn!("Waiting for weidu process failed");
            continue;
        }
        let result: ChildStdout = output.stdout.ok_or("Failed to get output ")?;
        let mut buffered_reader = BufReader::new(result);
        let mut buff = vec![];
        buffered_reader.read_to_end(&mut buff)?;
        let weidu_output = String::from_utf8(buff)?;
        log::debug!("{}", weidu_output);
        if weidu_output.is_empty() {
            log::warn!("Empty weidu response, {:?}", output.stderr);
        }
        weidu_output
            .split("\n")
            .filter(|x| (*x).starts_with(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']))
            .filter(|lang| {
                lang.to_lowercase()
                    .contains(&command.filter_by_selected_language)
            })
            .for_each(|lang| {
                log::debug!("{}", lang);
                if let Some((lang_num, _)) = lang.split_once(":") {
                    mod_langs.insert(lang_num);
                }
            });

        for mod_lang in mod_langs {
            let weidu_args = generate_args_for_list_components_with_game_dir(
                weidu_mod.to_str().unwrap_or_default(),
                mod_lang,
                command.game_directory.to_str().unwrap_or_default(),
            );
            log::debug!("{:?}", weidu_args);
            let mut run = Command::new(command.options.weidu_binary.clone());
            let output = run
                .current_dir(&command.game_directory)
                .args(weidu_args)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .current_dir(command.game_directory.clone())
                .spawn()?;
            if let Some(result) = output.stdout {
                let mut buffered_reader = BufReader::new(result);
                let mut buff = vec![];
                buffered_reader.read_to_end(&mut buff)?;
                let weidu_output = String::from_utf8(buff).unwrap_or_default();
                log::debug!("{}", weidu_output);
                weidu_output
                    .split("\n")
                    .filter(|x| (*x).starts_with("~"))
                    .for_each(|lang| println!("{}", shorten_weidu_component_path_string(lang)))
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn shortens_component_string_correctly() -> Result<(), Box<dyn Error>> {
        let test: &'static str = "~/HOME/TEST/.LOCAL/SHARE/STEAM/STEAMAPPS/COMMON/BG2/A7#IMPROVEDARCHER/A7#IMPROVEDARCHER.TP2~ #1 #10";
        let expected = "~A7#IMPROVEDARCHER/A7#IMPROVEDARCHER.TP2~ #1 #10";
        assert_eq!(shorten_weidu_component_path_string(test), expected);
        Ok(())
    }
}

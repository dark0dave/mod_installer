use std::collections::HashSet;
use std::io::{BufReader, Read};
use std::{error::Error, path::PathBuf, process::ChildStdout, process::Command, process::Stdio};

use walkdir::WalkDir;

use crate::config::args::Scan;

fn generate_args_for_list_components(mod_path: &str, lang: &str) -> Vec<String> {
    vec![
        "--nogame".to_string(),
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

pub(crate) fn scan(command: &Scan) -> Result<(), Box<dyn Error>> {
    let mods = find_mods(
        command.options.mod_directories.clone(),
        command.options.depth,
    );

    for weidu_mod in mods {
        let mut mod_langs: HashSet<&str> = HashSet::new();
        let weidu_args_langs = generate_args_for_list_lang(weidu_mod.to_str().unwrap_or_default());
        let mut run = Command::new(command.options.weidu_binary.clone());
        let mut ouput = run
            .args(weidu_args_langs)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Could not run");
        if let Err(_) = ouput.try_wait() {
            continue;
        }
        let result: ChildStdout = ouput.stdout.ok_or("Failed to get output ")?;
        let mut buffered_reader = BufReader::new(result);
        let mut buff = vec![];
        buffered_reader.read_to_end(&mut buff)?;
        let weidu_output = String::from_utf8(buff)?;
        // Drop weidu version information
        if let Some((_, tail)) = weidu_output.split_once("\n") {
            tail.split("\n")
                .filter(|x| !x.is_empty() && !x.starts_with("FATAL ERROR"))
                .for_each(|lang| {
                    // println!("{}", lang);
                    if let Some((lang_num, _)) = lang.split_once(":") {
                        mod_langs.insert(lang_num);
                    }
                });
        }
        for mod_lang in mod_langs {
            let weidu_args =
                generate_args_for_list_components(weidu_mod.to_str().unwrap_or_default(), mod_lang);
            log::debug!("{:?}", weidu_args);
            let mut run = Command::new(command.options.weidu_binary.clone());
            let ouput = run
                .current_dir(command.game_directory.to_path_buf())
                .args(weidu_args)
                .spawn()?;
            if let Some(out) = ouput.stdout {
                log::info!("{:?}", out);
            }
        }
    }
    Ok(())
}

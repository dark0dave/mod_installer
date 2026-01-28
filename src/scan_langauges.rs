use std::collections::HashSet;
use std::io::{BufReader, Read};
use std::path::Path;
use std::{error::Error, process::ChildStdout, process::Command, process::Stdio};

use config::args::ScanLangauges;

use crate::utils::find_all_mods;

fn generate_args_for_list_lang(mod_path: &str) -> Vec<String> {
    vec![
        "--nogame".to_string(),
        "--list-languages".to_string(),
        mod_path.to_string(),
        "--no-exit-pause".to_string(),
    ]
}

pub(crate) fn scan_for_langauges(
    weidu_mod: &str,
    weidu_binary: &Path,
    filter_by_selected_language: &str,
) -> Result<HashSet<String>, Box<dyn Error>> {
    let weidu_args_langs = generate_args_for_list_lang(weidu_mod);
    let mut run = Command::new(weidu_binary);
    let mut output = run
        .args(weidu_args_langs)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Err(err) = output.try_wait() {
        log::warn!("Waiting for weidu process failed");
        return Err(Box::new(err));
    }
    let result: ChildStdout = output.stdout.ok_or("Failed to get output")?;
    let mut buffered_reader = BufReader::new(result);
    let mut buff = vec![];
    buffered_reader.read_to_end(&mut buff)?;
    let weidu_output = String::from_utf8(buff)?;
    log::trace!("{}", weidu_output);

    if weidu_output.is_empty() {
        log::warn!("Empty weidu response, {:?}", output.stderr);
        return Ok(HashSet::new());
    }
    Ok(weidu_output
        .split("\n")
        .flat_map(|lang| {
            if (*lang).starts_with(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'])
                && (*lang)
                    .to_lowercase()
                    .contains(&filter_by_selected_language.to_lowercase())
            {
                log::debug!("{}", lang);
                if let Some((lang_num, _)) = lang.split_once(":") {
                    return Some(lang_num.to_string());
                }
            }
            None
        })
        .collect())
}

pub(crate) fn scan_langauges(command: &ScanLangauges) -> Result<(), Box<dyn Error>> {
    let mods = find_all_mods(&command.options.mod_directories, command.options.depth);
    log::trace!("{:?}", mods);

    for weidu_mod in mods {
        let langs = scan_for_langauges(
            weidu_mod.to_str().unwrap_or_default(),
            &command.options.weidu_binary,
            &command.filter_by_selected_language,
        );
        println!("{:?} {:?}", weidu_mod, langs)
    }
    Ok(())
}

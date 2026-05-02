use std::ffi::{OsStr, OsString};
use std::io::{BufReader, Read};
use std::{error::Error, process::Command, process::Stdio};

use crate::config::args::ScanComponents;
use crate::scan::languages::scan_for_langauges;
use crate::utils::find_all_mods;
use crate::weidu::component::WeiduComponent;

fn generate_args_for_list_components_with_game_dir(
    mod_path: &OsStr,
    lang: &str,
    game_dir: &OsStr,
) -> Vec<OsString> {
    vec![
        "--game".into(),
        game_dir.into(),
        "--list-components".into(),
        mod_path.into(),
        lang.into(),
        "--no-exit-pause".into(),
    ]
}

pub(crate) fn scan_components(command: &ScanComponents) -> Result<(), Box<dyn Error>> {
    let mod_paths = find_all_mods(&command.options.mod_directories, command.options.depth);
    log::trace!("{:?}", mod_paths);

    for (_, mod_path) in mod_paths {
        let mod_langs = scan_for_langauges(
            mod_path.as_os_str(),
            &command.options.weidu_binary,
            &command.filter_by_selected_language,
        )?;
        for mod_lang in mod_langs {
            let weidu_args = generate_args_for_list_components_with_game_dir(
                mod_path.as_os_str(),
                &mod_lang,
                command.game_directory.as_os_str(),
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
                    .flat_map(|comp| WeiduComponent::try_from(comp.to_string()))
                    .for_each(|comp| println!("{:?}", comp))
            }
        }
    }
    Ok(())
}

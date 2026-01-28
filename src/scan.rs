use std::io::{BufReader, Read};
use std::{error::Error, process::Command, process::Stdio};

use config::args::ScanComponents;

use crate::component::Component;
use crate::scan_langauges::scan_for_langauges;
use crate::utils::find_all_mods;

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

pub(crate) fn scan_components(command: &ScanComponents) -> Result<(), Box<dyn Error>> {
    let mods = find_all_mods(&command.options.mod_directories, command.options.depth);
    log::trace!("{:?}", mods);

    for weidu_mod in mods {
        let mod_langs = scan_for_langauges(
            weidu_mod.to_str().unwrap_or_default(),
            &command.options.weidu_binary,
            &command.filter_by_selected_language,
        )?;
        for mod_lang in mod_langs {
            let weidu_args = generate_args_for_list_components_with_game_dir(
                weidu_mod.to_str().unwrap_or_default(),
                &mod_lang,
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
                    .flat_map(|comp| Component::try_from(comp.to_string()))
                    .for_each(|comp| println!("{:?}", comp))
            }
        }
    }
    Ok(())
}

use core::time;
use fs_extra::dir::{copy, CopyOptions};
use std::{
    error::Error,
    path::{Path, PathBuf},
    thread,
};
use walkdir::WalkDir;

use crate::{component::Component, log_file::LogFile};

pub fn mod_folder_present_in_game_directory(game_directory: &Path, mod_name: &str) -> bool {
    game_directory.join(mod_name).is_dir()
}

pub fn copy_mod_folder(game_directory: &Path, mod_folder: &Path) {
    let mut options = CopyOptions::new();
    options.skip_exist = true;
    let copied = copy(mod_folder, game_directory, &options);
    if let Err(error) = copied {
        log::error!("Failed to copy mod {:?} with error: {}", mod_folder, error);
        panic!()
    }
}

pub fn search_mod_folders(
    folder_directories: &[PathBuf],
    weidu_mod: &Component,
    depth: usize,
) -> PathBuf {
    let mod_folder_locations = folder_directories
        .iter()
        .find_map(|mod_folder| find_mod_folder(weidu_mod, mod_folder, depth));

    if let Some(mod_folder) = mod_folder_locations {
        mod_folder
    } else {
        log::error!("Could not find {:#?} mod folder ", weidu_mod);
        panic!()
    }
}

fn find_mod_folder(mod_component: &Component, mod_dir: &Path, depth: usize) -> Option<PathBuf> {
    WalkDir::new(mod_dir)
        .follow_links(true)
        .max_depth(depth)
        .into_iter()
        .find_map(|entry| match entry {
            Ok(entry)
                if entry
                    .path()
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap_or_default()
                    .eq_ignore_ascii_case(&mod_component.name)
                    && entry
                        .file_name()
                        .eq_ignore_ascii_case(&mod_component.tp_file) =>
            {
                return Some(entry.into_path().parent().unwrap().into());
            }
            _ => None,
        })
}

pub(crate) fn find_mods(
    log_file: PathBuf,
    skip_installed: bool,
    game_directory: PathBuf,
    strict_matching: bool,
) -> Result<LogFile, Box<dyn Error>> {
    let mut mods = LogFile::try_from(log_file)?;
    let number_of_mods_found = mods.len();
    let mods_to_be_installed = if skip_installed {
        let existing_weidu_log_file_path = game_directory.join("weidu").with_extension("log");
        if let Ok(installed_mods) = LogFile::try_from(existing_weidu_log_file_path) {
            for installed_mod in &installed_mods {
                if strict_matching {
                    mods.retain(|mod_to_install| installed_mod.strict_matching(mod_to_install));
                } else {
                    mods.retain(|mod_to_install| installed_mod != mod_to_install);
                }
            }
        }
        mods
    } else {
        mods
    };

    log::info!(
        "Number of mods found: {}, Number of mods to be installed: {}",
        number_of_mods_found,
        mods_to_be_installed.len()
    );
    Ok(mods_to_be_installed)
}

pub fn sleep(millis: u64) {
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn finds_mod_folder() -> Result<(), Box<dyn Error>> {
        let mod_component = Component {
            tp_file: "TEST.TP2".to_string(),
            name: "test_mod_name_1".to_string(),
            lang: "0".to_string(),
            component: "0".to_string(),
            component_name: "".to_string(),
            sub_component: "".to_string(),
            version: "".to_string(),
        };
        let mod_folder = find_mod_folder(&mod_component, Path::new("fixtures/mods"), 3);

        let expected =
            Path::new(&format!("fixtures/mods/mod_a/{}", mod_component.name)).to_path_buf();
        assert_eq!(mod_folder, Some(expected));
        Ok(())
    }

    #[test]
    fn test_find_mods() -> Result<(), Box<dyn Error>> {
        let log_file = PathBuf::from("./fixtures/test.log");
        let skip_installed = false;
        let game_directory = PathBuf::from("./fixtures");
        let result = find_mods(log_file.clone(), skip_installed, game_directory, false)?;
        let expected = LogFile::try_from(log_file)?;
        assert_eq!(expected, result);
        Ok(())
    }

    #[test]
    fn test_find_mods_skip_installed() -> Result<(), Box<dyn Error>> {
        let log_file = PathBuf::from("./fixtures/test.log");
        let skip_installed = true;
        let game_directory = PathBuf::from("./fixtures");
        let result = find_mods(log_file, skip_installed, game_directory, false)?;
        let expected = LogFile(vec![
            Component {
                tp_file: "TEST.TP2".to_string(),
                name: "test_mod_name_1".to_string(),
                lang: "0".to_string(),
                component: "1".to_string(),
                component_name: "test mod two".to_string(),
                sub_component: "".to_string(),
                version: "".to_string(),
            },
            Component {
                tp_file: "END.TP2".to_string(),
                name: "test_mod_name_3".to_string(),
                lang: "0".to_string(),
                component: "0".to_string(),
                component_name: "test mod with version".to_string(),
                sub_component: "".to_string(),
                version: "1.02".to_string(),
            },
        ]);
        assert_eq!(expected, result);
        Ok(())
    }
}

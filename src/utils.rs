use core::time;
use fs_extra::dir::{copy, CopyOptions};
use std::{
    path::{Path, PathBuf},
    thread,
};
use walkdir::WalkDir;

use crate::component::Component;

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

pub fn sleep(millis: u64) {
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn finds_mod_folder() {
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
        assert_eq!(mod_folder, Some(expected))
    }
}

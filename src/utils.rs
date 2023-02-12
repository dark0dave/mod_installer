use fs_extra::dir::{copy, CopyOptions};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn mod_folder_present_in_game_directory(game_directory: &Path, mod_name: &str) -> bool {
    game_directory.join(mod_name).is_dir()
}

pub fn copy_mod_folder(game_directory: &Path, mod_folder: &Path) {
    let mut options = CopyOptions::new();
    options.skip_exist = true;
    let coppied = copy(mod_folder, game_directory, &options);
    if let Err(error) = coppied {
        log::error!("Failed to copy with error: {}", error);
        panic!()
    }
}

pub fn find_mod_folder(mod_name: &str, mod_dir: &Path) -> Option<PathBuf> {
    WalkDir::new(mod_dir)
        .follow_links(true)
        .max_depth(2)
        .into_iter()
        .flat_map(|entry| {
            if let Ok(entry) = entry {
                if entry.file_type().is_dir() && entry.file_name().eq_ignore_ascii_case(mod_name) {
                    return Some(entry.into_path());
                }
            }
            None
        })
        .collect::<Vec<PathBuf>>()
        .first()
        .cloned()
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn finds_mod_folder() {
        let mod_name = "test_mod_name_1";
        let mod_folder = find_mod_folder(mod_name, &Path::new("fixtures/mods").to_path_buf());

        let expected = Path::new(&format!("fixtures/mods/mod_a/{}", mod_name)).to_path_buf();
        assert_eq!(mod_folder, Some(expected))
    }
}

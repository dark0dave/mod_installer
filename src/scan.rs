use std::{error::Error, path::PathBuf, process::Command};

use walkdir::WalkDir;

use crate::config::args::Scan;

fn generate_args(mod_path: &str, game_dir: &str, lang: &str) -> Vec<String> {
    vec![
        "--game".to_string(),
        game_dir.to_string(),
        "--list-components".to_string(),
        mod_path.to_string(),
        lang.to_string(),
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
        let weidu_args = generate_args(
            weidu_mod.to_str().unwrap_or_default(),
            command.game_directory.to_str().unwrap_or_default(),
            "0",
        );
        println!("{:?}", weidu_args);
        let mut run = Command::new(command.options.weidu_binary.clone());
        let ouput = run
            .current_dir(command.game_directory.to_path_buf())
            .args(weidu_args)
            .spawn()?;
        println!("{:?}", ouput.stdout.ok_or("Failed to get output ")?);
    }
    Ok(())
}

use std::path::{Path, PathBuf};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Full path to target log
    #[clap(env, long, required = true)]
    pub log_file: PathBuf,

    /// Full path to game directory
    #[clap(env, short, long, value_parser = parse_absolute_path, required = true)]
    pub game_directory: PathBuf,

    /// Full Path to weidu binary
    #[clap(env, short, long, value_parser = parse_absolute_path, required = true)]
    pub weidu_binary: PathBuf,

    /// Full Path to mod directories
    #[clap(
        env,
        short,
        long,
        value_parser = parse_absolute_path,
        use_value_delimiter = true,
        value_delimiter = ',',
        required = true
    )]
    pub mod_directories: Vec<PathBuf>,

    /// Game Language
    #[clap(short, long, default_value = "en_US")]
    pub language: String,
}

fn parse_absolute_path(arg: &str) -> Result<PathBuf, String> {
    let path = Path::new(arg);
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Err("Please provide the absolute path".to_string())
    }
}

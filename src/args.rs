use std::path::{Path, PathBuf};

use clap::{ArgAction, Parser};

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

    /// Depth to walk folder structure
    #[clap(env, long, short, default_value = "3")]
    pub depth: usize,

    /// Compare against installed weidu log, note this is best effort
    #[clap(env, long, short, action=ArgAction::SetTrue, default_value = "true")]
    pub skip_installed: bool,

    /// If a warning occurs in the weidu child process exit
    #[clap(env, long, short, action=ArgAction::SetTrue, default_value = "true")]
    pub abort_on_warnings: bool,

    /// Timeout time per mod in seconds, default is 1 hour
    #[clap(env, long, short, default_value = "3600")]
    pub timeout: usize,
}

fn parse_absolute_path(arg: &str) -> Result<PathBuf, String> {
    let path = Path::new(arg);
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Err("Please provide the absolute path".to_string())
    }
}

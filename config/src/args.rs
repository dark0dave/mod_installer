use std::env::{split_paths, var_os};
use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;

use clap::Subcommand;
use clap::{Parser, builder::BoolishValueParser, builder::OsStr};

use crate::log_options::LogOptions;

use super::colors::styles;

pub const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

pub const LONG: &str = r"

  /\/\   ___   __| | (_)_ __  ___| |_ __ _| | | ___ _ __
 /    \ / _ \ / _` | | | '_ \/ __| __/ _` | | |/ _ \ '__|
/ /\/\ \ (_) | (_| | | | | | \__ \ || (_| | | |  __/ |
\/    \/\___/ \__,_| |_|_| |_|___/\__\__,_|_|_|\___|_|
";

#[cfg(not(target_os = "windows"))]
pub const WEIDU_FILE_NAME: &str = "weidu";
#[cfg(target_os = "windows")]
pub const WEIDU_FILE_NAME: &str = "weidu.exe";

// https://docs.rs/clap/latest/clap/_derive/index.html#arg-attributes
#[derive(Parser, Debug, PartialEq)]
#[command(
    version,
    propagate_version = true,
    styles = styles(),
    about = format!("{}\n{}", LONG, std::env::var("CARGO_PKG_DESCRIPTION").unwrap_or_default())
)]
pub struct Args {
    /// Install Type
    #[command(subcommand)]
    pub command: CommandType,
}

/// Type of Install, Normal or EET
#[derive(Subcommand, Debug, PartialEq, Clone)]
pub enum CommandType {
    #[command()]
    Normal(Normal),
    #[command()]
    Eet(Eet),
    #[command(subcommand)]
    Scan(Scan),
}

/// Normal install for (BG1EE,BG2EE,IWDEE) (STABLE)
#[derive(Parser, Debug, PartialEq, Clone)]
#[clap(short_flag = 'n')]
pub struct Normal {
    /// Path to target log
    #[clap(env, long, short = 'f', value_parser = path_must_exist, required = true)]
    pub log_file: PathBuf,

    /// Absolute Path to game directory
    #[clap(env, short, long, value_parser = parse_absolute_path, required = true)]
    pub game_directory: PathBuf,

    /// Instead of operating on an existing directory, create a new one with this flag as its name and then copy the original contents into it.
    #[clap(env, long, short = 'n', required = false)]
    pub generate_directory: Option<PathBuf>,

    /// CommonOptions
    #[clap(flatten)]
    pub options: Options,
}

/// EET install for (eet) (BETA)
#[derive(Parser, Debug, PartialEq, Clone)]
#[clap(short_flag = 'e')]
pub struct Eet {
    /// Absolute Path to bg1ee game directory
    #[clap(env, short='1', long, value_parser = parse_absolute_path, required = true)]
    pub bg1_game_directory: PathBuf,

    /// Path to bg1ee weidu.log file
    #[clap(env, short='y', long, value_parser = path_must_exist, required = true)]
    pub bg1_log_file: PathBuf,

    /// Absolute Path to bg2ee game directory
    #[clap(env, short='2', long, value_parser = parse_absolute_path, required = true)]
    pub bg2_game_directory: PathBuf,

    /// Path to bg2ee weidu.log file
    #[clap(env, short='z', long, value_parser = path_must_exist, required = true)]
    pub bg2_log_file: PathBuf,

    /// Generates a new pre-eet directory.
    #[clap(env, short = 'p', long, value_parser = path_must_exist)]
    pub new_pre_eet_dir: Option<PathBuf>,

    /// Generates a new eet directory.
    #[clap(env, short = 'n', long, value_parser = path_must_exist)]
    pub new_eet_dir: Option<PathBuf>,

    /// CommonOptions
    #[clap(flatten)]
    pub options: Options,
}

/// Scan for (BG1EE,BG2EE,IWDEE) (ALPHA)
#[derive(Subcommand, Debug, PartialEq, Clone)]
#[clap(short_flag = 's')]
pub enum Scan {
    #[command()]
    Langauges(ScanLangauges),
    #[command()]
    Components(ScanComponents),
}

#[derive(Parser, Debug, PartialEq, Clone)]
#[clap(short_flag = 'l')]
pub struct ScanLangauges {
    /// filter by selected language
    #[clap(short, long, required = false, default_value = "")]
    pub filter_by_selected_language: String,

    /// CommonOptions
    #[clap(flatten)]
    pub options: Options,
}

#[derive(Parser, Debug, PartialEq, Clone)]
#[clap(short_flag = 'c')]
pub struct ScanComponents {
    /// Absolute Path to game directory
    #[clap(env, short, long, value_parser = parse_absolute_path)]
    pub game_directory: PathBuf,

    /// filter by selected language
    #[clap(short, long, required = false, default_value = "")]
    pub filter_by_selected_language: String,

    /// CommonOptions
    #[clap(flatten)]
    pub options: Options,
}

#[derive(Parser, Debug, PartialEq, Clone, Default)]
pub struct Options {
    /// Absolute Path to weidu binary
    #[clap(
        env,
        short,
        long,
        value_parser = parse_absolute_path,
        default_value_os_t = find_weidu_bin(),
        default_missing_value = find_weidu_bin_on_path(),
        required = false
    )]
    pub weidu_binary: PathBuf,

    /// Path to mod directories
    #[clap(
        env,
        short,
        long,
        value_parser = path_exists_full,
        use_value_delimiter = true,
        value_delimiter = ',',
        default_values_os_t = current_work_dir(),
        default_missing_value = working_dir(),
        required = false
    )]
    pub mod_directories: Vec<PathBuf>,

    /// Game Language
    #[clap(short, long, default_value = "en_US")]
    pub language: String,

    /// Depth to walk folder structure
    #[clap(env, long, short, default_value = "5")]
    pub depth: usize,

    /// Compare against installed weidu log, note this is best effort
    #[clap(
        env,
        short,
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetFalse,
        default_value_t = true,
        value_parser = BoolishValueParser::new(),
    )]
    pub skip_installed: bool,

    /// If a warning occurs in the weidu child process exit
    #[clap(
        env,
        short,
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetTrue,
        default_value_t = false,
        value_parser = BoolishValueParser::new(),
    )]
    pub abort_on_warnings: bool,

    /// Timeout time per mod in seconds, default is 1 hour
    #[clap(env, long, short, default_value = "3600")]
    pub timeout: usize,

    /// Weidu log setting "autolog,logapp,log-extern" is default
    #[clap(
        env,
        long,
        short='u',
        use_value_delimiter = true,
        value_delimiter = ',',
        default_value = "autolog,logapp,log-extern",
        value_parser = LogOptions::value_parser,
        required = false
    )]
    pub weidu_log_mode: Vec<LogOptions>,

    /// Strict Version and Component/SubComponent matching
    #[clap(
        env,
        short = 'x',
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetTrue,
        default_value_t = false,
        value_parser = BoolishValueParser::new(),
    )]
    pub strict_matching: bool,

    /// When a missing log is discovered ask the user for the download uri, download the mod and install it
    #[clap(
        env,
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetFalse,
        default_value_t = true,
        value_parser = BoolishValueParser::new(),
    )]
    pub download: bool,

    /// Force copy mod folder, even if the mod folder was found in the game directory
    #[clap(
        env,
        short,
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetTrue,
        default_value_t = false,
        value_parser = BoolishValueParser::new(),
    )]
    pub overwrite: bool,

    /// Strict weidu log checking
    #[clap(
        env,
        short,
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetFalse,
        default_value_t = true,
        value_parser = BoolishValueParser::new(),
    )]
    pub check_last_installed: bool,

    /// Tick
    #[clap(env, short = 'i', long, default_value_t = 500)]
    pub tick: u64,
}

pub fn path_must_exist(arg: &str) -> Result<PathBuf, std::io::Error> {
    let path = PathBuf::from(arg);
    path.try_exists()?;
    Ok(path)
}

fn path_exists_full(arg: &str) -> Result<PathBuf, std::io::Error> {
    fs::canonicalize(path_must_exist(arg)?)
}

fn parse_absolute_path(arg: &str) -> Result<PathBuf, String> {
    let path = path_must_exist(arg).map_err(|err| err.to_string())?;
    fs::canonicalize(path).map_err(|err| err.to_string())
}

fn find_weidu_bin() -> PathBuf {
    if let Some(paths) = var_os("PATH") {
        for path in split_paths(&paths) {
            let full_path = path.join(WEIDU_FILE_NAME);
            if full_path.is_file() && !full_path.is_dir() {
                return full_path;
            }
        }
    }
    PathBuf::new()
}

fn find_weidu_bin_on_path() -> OsStr {
    OsStr::from(find_weidu_bin().into_os_string())
}

fn current_work_dir() -> Vec<PathBuf> {
    vec![std::env::current_dir().unwrap()]
}

fn working_dir() -> OsStr {
    OsStr::from(current_work_dir().first().unwrap().clone().into_os_string())
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;
    use std::{error::Error, path::PathBuf};

    #[test]
    fn test_bool_flags() -> Result<(), Box<dyn Error>> {
        let workspace_root: PathBuf = std::env::current_dir()?;
        let fake_game_dir: PathBuf = workspace_root
            .parent()
            .ok_or("Could not get workspace root")?
            .join("fixtures");
        let fake_weidu_bin = fake_game_dir.clone().join("weidu");
        let fake_log_file = fake_game_dir.clone().join("weidu.log");
        let fake_mod_dirs = fake_game_dir.clone().join("mods");
        let tests = vec![
            ("true", true),
            ("t", true),
            ("yes", true),
            ("y", true),
            ("1", true),
            ("false", false),
            ("f", false),
            ("no", false),
            ("n", false),
            ("0", false),
        ];
        for (flag_value, expected_flag_value) in tests {
            let expected = Args {
                command: CommandType::Normal(Normal {
                    log_file: fake_log_file.clone(),
                    game_directory: fake_game_dir.clone(),
                    generate_directory: None,
                    options: Options {
                        weidu_binary: fake_weidu_bin.clone(),
                        mod_directories: vec![fake_mod_dirs.clone()],
                        language: "en_US".to_string(),
                        depth: 5,
                        skip_installed: expected_flag_value,
                        abort_on_warnings: expected_flag_value,
                        timeout: 3600,
                        weidu_log_mode: vec![
                            LogOptions::AutoLog,
                            LogOptions::LogAppend,
                            LogOptions::LogExternal,
                        ],
                        strict_matching: true,
                        download: true,
                        overwrite: false,
                        check_last_installed: true,
                        tick: 500,
                    },
                }),
            };
            let test_arg_string = format!(
                "mod_installer -n -x -a {} -s {} -w {} -m {} -f {} -g {}",
                flag_value,
                flag_value,
                fake_weidu_bin.to_str().unwrap_or_default(),
                fake_mod_dirs.to_str().unwrap_or_default(),
                fake_log_file.to_str().unwrap_or_default(),
                fake_game_dir.to_str().unwrap_or_default(),
            );
            let result = Args::parse_from(test_arg_string.split(' '));
            assert_eq!(
                result, expected,
                "Result {result:?} didn't match Expected {expected:?}",
            );
        }
        Ok(())
    }

    #[test]
    fn test_eet_flags() -> Result<(), Box<dyn Error>> {
        let workspace_root: PathBuf = std::env::current_dir()?;
        let fake_game_dir: PathBuf = workspace_root
            .parent()
            .ok_or("Could not get workspace root")?
            .join("fixtures");
        let fake_weidu_bin = fake_game_dir.clone().join("weidu");
        let fake_log_file = fake_game_dir.clone().join("weidu.log");
        let new_dir = PathBuf::new().join("test");
        let expected_flag_value = true;

        let expected = Args {
            command: CommandType::Eet(Eet {
                bg1_game_directory: fake_game_dir.clone(),
                bg1_log_file: fake_log_file.clone(),
                bg2_game_directory: fake_game_dir.clone(),
                bg2_log_file: fake_log_file.clone(),
                options: Options {
                    weidu_binary: fake_weidu_bin.clone(),
                    mod_directories: vec![std::env::current_dir().unwrap()],
                    language: "en_US".to_string(),
                    depth: 5,
                    skip_installed: expected_flag_value,
                    abort_on_warnings: !expected_flag_value,
                    timeout: 3600,
                    weidu_log_mode: vec![
                        LogOptions::AutoLog,
                        LogOptions::LogAppend,
                        LogOptions::LogExternal,
                    ],
                    strict_matching: !expected_flag_value,
                    download: expected_flag_value,
                    overwrite: !expected_flag_value,
                    check_last_installed: expected_flag_value,
                    tick: 500,
                },
                new_pre_eet_dir: None,
                new_eet_dir: Some("test".into()),
            }),
        };
        let test_arg_string = format!(
            "mod_installer eet -w {} -1 {} -y {} -2 {} -z {} -n {}",
            fake_weidu_bin.to_str().unwrap_or_default(),
            fake_game_dir.to_str().unwrap_or_default(),
            fake_log_file.to_str().unwrap_or_default(),
            fake_game_dir.to_str().unwrap_or_default(),
            fake_log_file.to_str().unwrap_or_default(),
            new_dir.to_str().unwrap_or_default(),
        );
        let result = Args::parse_from(test_arg_string.split(' '));
        assert_eq!(
            result, expected,
            "Result {result:?} didn't match Expected {expected:?}",
        );

        Ok(())
    }
}

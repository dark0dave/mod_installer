use std::env::{split_paths, var_os};
use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;

use clap::Subcommand;
use clap::{builder::BoolishValueParser, builder::OsStr, Parser};

use super::colors::styles;

pub(crate) const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

pub(crate) const LONG: &str = r"

  /\/\   ___   __| | (_)_ __  ___| |_ __ _| | | ___ _ __
 /    \ / _ \ / _` | | | '_ \/ __| __/ _` | | |/ _ \ '__|
/ /\/\ \ (_) | (_| | | | | | \__ \ || (_| | | |  __/ |
\/    \/\___/ \__,_| |_|_| |_|___/\__\__,_|_|_|\___|_|
";

pub(crate) const WEIDU_LOG_MODE_ERROR: &str = r"
Please provide a valid weidu logging setting, options are:
--weidu-log-mode log X       log output and details to X
--weidu-log-mode autolog     log output and details to WSETUP.DEBUG
--weidu-log-mode logapp      append to log instead of overwriting
--weidu-log-mode log-extern  also log output from commands invoked by WeiDU
";

#[cfg(not(target_os = "windows"))]
pub(crate) const WEIDU_FILE_NAME: &str = "weidu";
#[cfg(target_os = "windows")]
pub(crate) const WEIDU_FILE_NAME: &str = "weidu.exe";

// https://docs.rs/clap/latest/clap/_derive/index.html#arg-attributes
#[derive(Parser, Debug, PartialEq)]
#[command(
    version,
    propagate_version = true,
    styles = styles(),
    about = format!("{}\n{}", LONG, std::env::var("CARGO_PKG_DESCRIPTION").unwrap_or_default())
)]
pub(crate) struct Args {
    /// Install Type
    #[command(subcommand)]
    pub(crate) command: InstallType,
}

/// Type of Install, Normal or EET
#[derive(Subcommand, Debug, PartialEq, Clone)]
pub(crate) enum InstallType {
    #[command()]
    Normal(Normal),
    #[command()]
    Eet(Eet),
}

/// Normal install for (BG1EE,BG2EE,IWDEE) (STABLE)
#[derive(Parser, Debug, PartialEq, Clone)]
#[clap(short_flag = 'n')]
pub(crate) struct Normal {
    /// Path to target log
    #[clap(env, long, short = 'f', value_parser = path_must_exist, required = true)]
    pub(crate) log_file: PathBuf,

    /// Absolute Path to game directory
    #[clap(env, short, long, value_parser = parse_absolute_path, required = true)]
    pub(crate) game_directory: PathBuf,

    /// Instead of operating on an existing directory, create a new one with this flag as its name and then copy the original contents into it.
    #[clap(env, long, short = 'n', required = false)]
    pub(crate) new_game_directory: Option<PathBuf>,

    /// CommonOptions
    #[clap(flatten)]
    pub(crate) options: Options,
}

/// EET install for (eet) (ALPHA)
#[derive(Parser, Debug, PartialEq, Clone)]
#[clap(short_flag = 'e')]
pub(crate) struct Eet {
    /// Absolute Path to bg1ee game directory
    #[clap(env, short='1', long, value_parser = parse_absolute_path, required = true)]
    pub(crate) bg1_game_directory: PathBuf,

    /// Path to bg1ee weidu.log file
    #[clap(env, short='y', long, value_parser = path_must_exist, required = true)]
    pub(crate) bg1_log_file: PathBuf,

    /// Absolute Path to bg2ee game directory
    #[clap(env, short='2', long, value_parser = parse_absolute_path, required = true)]
    pub(crate) bg2_game_directory: PathBuf,

    /// Path to bg2ee weidu.log file
    #[clap(env, short='z', long, value_parser = path_must_exist, required = true)]
    pub(crate) bg2_log_file: PathBuf,

    /// Instead of operating on an existing directory, create a new one with this flag as its name and then copy the original contents into it.
    #[clap(
        env,
        short,
        long,
        action = clap::ArgAction::SetTrue,
        required = false,
    )]
    pub(crate) generate_directories: bool,

    /// If generate_directories is true. This is the path for the new pre-eet directory.
    #[clap(env, short = 'p', long, value_parser = path_must_exist)]
    pub(crate) new_pre_eet_dir: Option<PathBuf>,

    /// If generate_directories is true. This is the path for the new eet directory.
    #[clap(env, short = 'n', long, value_parser = path_must_exist)]
    pub(crate) new_eet_dir: Option<PathBuf>,

    /// CommonOptions
    #[clap(flatten)]
    pub(crate) options: Options,
}

#[derive(Parser, Debug, PartialEq, Clone)]
pub(crate) struct Options {
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
    pub(crate) weidu_binary: PathBuf,

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
    pub(crate) mod_directories: Vec<PathBuf>,

    /// Game Language
    #[clap(short, long, default_value = "en_US")]
    pub(crate) language: String,

    /// Depth to walk folder structure
    #[clap(env, long, short, default_value = "5")]
    pub(crate) depth: usize,

    /// Compare against installed weidu log, note this is best effort
    #[clap(
        env,
        short,
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetFalse,
        default_value_t = true,
        default_missing_value = "true",
        value_parser = BoolishValueParser::new(),
    )]
    pub(crate) skip_installed: bool,

    /// If a warning occurs in the weidu child process exit
    #[clap(
        env,
        short,
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetTrue,
        default_value_t = false,
        default_missing_value = "false",
        value_parser = BoolishValueParser::new(),
    )]
    pub(crate) abort_on_warnings: bool,

    /// Timeout time per mod in seconds, default is 1 hour
    #[clap(env, long, short, default_value = "3600")]
    pub(crate) timeout: usize,

    /// Weidu log setting "--autolog" is default
    #[clap(
        env,
        long,
        short='u',
        default_value = "autolog",
        value_parser = parse_weidu_log_mode,
        required = false
    )]
    pub(crate) weidu_log_mode: String,

    /// Strict Version and Component/SubComponent matching
    #[clap(
        env,
        short = 'x',
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetTrue,
        default_value_t = false,
        default_missing_value = "false",
        value_parser = BoolishValueParser::new(),
        required = false,
    )]
    pub(crate) strict_matching: bool,

    /// When a missing log is discovered ask the user for the download uri, download the mod and install it
    #[clap(
        env,
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetFalse,
        default_value_t = true,
        default_missing_value = "true",
        value_parser = BoolishValueParser::new(),
    )]
    pub(crate) download: bool,

    /// Force copy mod folder, even if the mod folder was found in the game directory
    #[clap(
        env,
        short,
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetTrue,
        default_value_t = false,
        default_missing_value = "false",
        value_parser = BoolishValueParser::new(),
    )]
    pub(crate) overwrite: bool,
}

fn parse_weidu_log_mode(arg: &str) -> Result<String, String> {
    let mut args = arg.split(' ');
    let mut output = vec![];
    while let Some(arg) = args.next() {
        match arg {
            "log" if path_must_exist(arg).is_ok() => {
                let path = args.next().unwrap();
                output.push(format!("--{arg} {path}"));
            }
            "autolog" => output.push(format!("--{arg}")),
            "logapp" => output.push(format!("--{arg}")),
            "log-extern" => output.push(format!("--{arg}")),
            _ => return Err(format!("{WEIDU_LOG_MODE_ERROR}, Provided {arg}")),
        };
    }
    Ok(output.join(" "))
}

fn path_must_exist(arg: &str) -> Result<PathBuf, std::io::Error> {
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
    use std::error::Error;

    #[test]
    fn test_parse_weidu_log_mode() -> Result<(), Box<dyn Error>> {
        let tests = vec![
            ("autolog", Ok("--autolog".to_string())),
            ("log /home", Ok("--log /home".to_string())),
            ("autolog logapp", Ok("--autolog --logapp".to_string())),
            (
                "autolog logapp log-extern",
                Ok("--autolog --logapp --log-extern".to_string()),
            ),
            (
                "log /home logapp log-extern",
                Ok("--log /home --logapp --log-extern".to_string()),
            ),
            (
                "fish",
                Err(format!("{}, Provided {}", WEIDU_LOG_MODE_ERROR, "fish")),
            ),
            (
                "log /home fish",
                Err(format!("{}, Provided {}", WEIDU_LOG_MODE_ERROR, "fish")),
            ),
        ];
        for (test, expected) in tests {
            println!("{test:#?}");
            let result = parse_weidu_log_mode(test);
            assert_eq!(
                result, expected,
                "Result {result:?} didn't match Expected {expected:?}",
            );
        }
        Ok(())
    }

    #[test]
    fn test_bool_flags() -> Result<(), Box<dyn Error>> {
        let fake_game_dir = std::env::current_dir().unwrap().join("fixtures");
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
                command: InstallType::Normal(Normal {
                    log_file: fake_log_file.clone(),
                    game_directory: fake_game_dir.clone(),
                    new_game_directory: None,
                    options: Options {
                        weidu_binary: fake_weidu_bin.clone(),
                        mod_directories: vec![fake_mod_dirs.clone()],
                        language: "en_US".to_string(),
                        depth: 5,
                        skip_installed: expected_flag_value,
                        abort_on_warnings: expected_flag_value,
                        timeout: 3600,
                        weidu_log_mode: "--autolog".to_string(),
                        strict_matching: false,
                        download: true,
                        overwrite: false,
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
        let fake_game_dir = std::env::current_dir().unwrap().join("fixtures");
        let fake_weidu_bin = fake_game_dir.clone().join("weidu");
        let fake_log_file = fake_game_dir.clone().join("weidu.log");
        let new_dir = PathBuf::new().join("test");
        let expected_flag_value = true;

        let expected = Args {
            command: InstallType::Eet(Eet {
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
                    weidu_log_mode: "--autolog".to_string(),
                    strict_matching: !expected_flag_value,
                    download: expected_flag_value,
                    overwrite: !expected_flag_value,
                },
                generate_directories: false,
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

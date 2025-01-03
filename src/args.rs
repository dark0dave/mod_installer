use std::path::PathBuf;

use clap::{builder::BoolishValueParser, Parser};

pub(crate) const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

pub(crate) const WEIDU_LOG_MODE_ERROR: &str = r"
Please provide a valid weidu logging setting, options are:
--weidu-log-mode log X       log output and details to X
--weidu-log-mode autolog     log output and details to WSETUP.DEBUG
--weidu-log-mode logapp      append to log instead of overwriting
--weidu-log-mode log-extern  also log output from commands invoked by WeiDU
";

#[derive(Parser, Debug, PartialEq)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Path to target log
    #[clap(env, long, short = 'f', value_parser = path_must_exist, required = true)]
    pub log_file: PathBuf,

    /// Absolute Path to game directory
    #[clap(env, short, long, value_parser = parse_absolute_path, required = true)]
    pub game_directory: PathBuf,

    /// Absolute Path to weidu binary
    #[clap(env, short, long, value_parser = parse_absolute_path, required = true)]
    pub weidu_binary: PathBuf,

    /// Path to mod directories
    #[clap(
        env,
        short,
        long,
        value_parser = path_must_exist,
        use_value_delimiter = true,
        value_delimiter = ',',
        required = true
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
        action = clap::ArgAction::Set,
        default_value_t = true,
        default_missing_value = "true",
        value_parser = BoolishValueParser::new(),
    )]
    pub skip_installed: bool,

    /// If a warning occurs in the weidu child process exit
    #[clap(
        env,
        short,
        long,
        num_args=0..=1,
        action = clap::ArgAction::Set,
        default_value_t = true,
        default_missing_value = "true",
        value_parser = BoolishValueParser::new(),
    )]
    pub abort_on_warnings: bool,

    /// Timeout time per mod in seconds, default is 1 hour
    #[clap(env, long, short, default_value = "3600")]
    pub timeout: usize,

    /// Weidu log setting "--autolog" is default
    #[clap(env, long, short='u', default_value = "autolog", value_parser = parse_weidu_log_mode, required = false)]
    pub weidu_log_mode: String,

    /// Strict Version and Component/SubComponent matching
    #[clap(
        env,
        short = 'x',
        long,
        num_args=0..=1,
        action = clap::ArgAction::Set,
        default_value_t = false,
        default_missing_value = "false",
        value_parser = BoolishValueParser::new(),
    )]
    pub strict_matching: bool,
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
            _ => return Err(format!("{}, Provided {}", WEIDU_LOG_MODE_ERROR, arg)),
        };
    }
    Ok(output.join(" "))
}

fn path_must_exist(arg: &str) -> Result<PathBuf, std::io::Error> {
    let path = PathBuf::from(arg);
    path.try_exists()?;
    Ok(path)
}

fn parse_absolute_path(arg: &str) -> Result<PathBuf, String> {
    let path = path_must_exist(arg).map_err(|err| err.to_string())?;
    if path.is_absolute() {
        Ok(path)
    } else {
        Err("Please provide an absolute path".to_string())
    }
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
                log_file: fake_log_file.clone(),
                game_directory: fake_game_dir.clone(),
                weidu_binary: fake_weidu_bin.clone(),
                mod_directories: vec![fake_mod_dirs.clone()],
                language: "en_US".to_string(),
                depth: 5,
                skip_installed: expected_flag_value,
                abort_on_warnings: expected_flag_value,
                timeout: 3600,
                weidu_log_mode: "--autolog".to_string(),
                strict_matching: expected_flag_value,
            };
            let test_arg_string = format!(
                "mod_installer -x {} -a {} -s {} -w {} -f {} -g {} -m {}",
                flag_value,
                flag_value,
                flag_value,
                fake_weidu_bin.to_str().unwrap_or_default(),
                fake_log_file.to_str().unwrap_or_default(),
                fake_game_dir.to_str().unwrap_or_default(),
                fake_mod_dirs.to_str().unwrap_or_default()
            );
            let result = Args::parse_from(test_arg_string.split(' '));
            assert_eq!(
                result, expected,
                "Result {result:?} didn't match Expected {expected:?}",
            );
        }
        Ok(())
    }
}

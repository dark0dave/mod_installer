use std::path::PathBuf;

use clap::{ArgAction, Parser};

const WEIDU_LOG_MODE_ERROR: &str = r"
Please provide a valid weidu logging setting, options are:
--weidu-log-mode log X       log output and details to X
--weidu-log-mode autolog     log output and details to WSETUP.DEBUG
--weidu-log-mode logapp      append to log instead of overwriting
--weidu-log-mode log-extern  also log output from commands invoked by WeiDU
";

#[derive(Parser, Debug)]
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
    #[clap(env, long, short, action=ArgAction::SetTrue, default_value = "true")]
    pub skip_installed: bool,

    /// If a warning occurs in the weidu child process exit
    #[clap(env, long, short, action=ArgAction::SetTrue, default_value = "true")]
    pub abort_on_warnings: bool,

    /// Timeout time per mod in seconds, default is 1 hour
    #[clap(env, long, short, default_value = "3600")]
    pub timeout: usize,

    /// Weidu log setting "--autolog" is default
    #[clap(env, long, short='u', default_value = "autolog", value_parser = parse_weidu_log_mode, required = false)]
    pub weidu_log_mode: String,
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

    #[test]
    fn test_parse_weidu_log_mode() {
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
    }
}

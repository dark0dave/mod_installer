use std::{error::Error, path::PathBuf};

use crate::args::path_must_exist;

pub const PATH_PROVIDED_ERROR: &str = "Path provided, does not exist: ";

pub const WEIDU_LOG_MODE_ERROR: &str = r"
Please provide a valid weidu logging setting, options are:
--weidu-log-mode log X       log output and details to X
--weidu-log-mode autolog     log output and details to WSETUP.DEBUG
--weidu-log-mode logapp      append to log instead of overwriting
--weidu-log-mode log-extern  also log output from commands invoked by WeiDU
";

#[derive(Debug, PartialEq, Clone)]
pub enum LogOptions {
    Log(PathBuf),
    AutoLog,
    LogAppend,
    LogExternal,
}

impl TryFrom<&str> for LogOptions {
    type Error = Box<dyn Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            x if x.starts_with("log ") => {
                if let Some((_, tail)) = value.split_once(' ') {
                    if let Ok(path) = path_must_exist(tail) {
                        #[cfg(not(target_os = "windows"))]
                        return Ok(LogOptions::Log(path.canonicalize()?));
                        #[cfg(windows)]
                        return Ok(LogOptions::Log(path));
                    }
                    return Err(format!("{} {:?}", PATH_PROVIDED_ERROR, tail).into());
                }
                Err(format!("{WEIDU_LOG_MODE_ERROR}, Provided {value}").into())
            }
            "autolog" => Ok(LogOptions::AutoLog),
            "logapp" => Ok(LogOptions::LogAppend),
            "log-extern" => Ok(LogOptions::LogExternal),
            _ => Err(format!("{WEIDU_LOG_MODE_ERROR}, Provided {value}").into()),
        }
    }
}

impl LogOptions {
    pub fn value_parser(arg: &str) -> Result<LogOptions, String> {
        LogOptions::try_from(arg).map_err(|err| err.to_string())
    }
    pub fn to_args(&self, path: &str) -> Vec<String> {
        match self {
            LogOptions::LogAppend => vec!["--logapp".to_string()],
            LogOptions::Log(path_buf) if path_buf.is_file() => vec![
                "--log".to_string(),
                path_buf
                    .as_os_str()
                    .to_str()
                    .unwrap_or_default()
                    .to_string(),
            ],
            LogOptions::Log(path_buf) => vec![
                "--log".to_string(),
                path_buf
                    .join(path)
                    .as_os_str()
                    .to_str()
                    .unwrap_or_default()
                    .to_string(),
            ],
            LogOptions::AutoLog => vec!["--autolog".to_string()],
            LogOptions::LogExternal => vec!["--log-extern".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_weidu_log_mode() -> Result<(), Box<dyn Error>> {
        let tests = vec![
            (
                "log log",
                Err("No such file or directory (os error 2)".to_string()),
            ),
            ("autolog", Ok(LogOptions::AutoLog)),
            ("logapp", Ok(LogOptions::LogAppend)),
            ("log-extern", Ok(LogOptions::LogExternal)),
            ("", Err(format!("{WEIDU_LOG_MODE_ERROR}, Provided "))),
        ];
        for (test, expected) in tests {
            let result = LogOptions::value_parser(test);
            assert_eq!(
                result, expected,
                "Result {result:?} didn't match Expected {expected:?}",
            );
        }
        Ok(())
    }
}

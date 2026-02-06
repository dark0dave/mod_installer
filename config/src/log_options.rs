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
                        return Ok(LogOptions::Log(path.canonicalize()?));
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

impl From<String> for LogOptions {
    fn from(value: String) -> Self {
        LogOptions::try_from(value.as_str()).unwrap()
    }
}

impl LogOptions {
    pub fn value_parser(arg: &str) -> Result<LogOptions, String> {
        LogOptions::try_from(arg).map_err(|err| err.to_string())
    }
    pub fn to_string(&self, path: &str) -> String {
        match self {
            LogOptions::Log(path_buf) if path_buf.is_file() => {
                format!("--log {}", path_buf.to_string_lossy())
            }
            LogOptions::Log(path_buf) => {
                format!("--log {}", path_buf.join(path).to_string_lossy())
            }
            LogOptions::AutoLog => "--autolog".to_string(),
            LogOptions::LogAppend => "--logapp".to_string(),
            LogOptions::LogExternal => "--log-extern".to_string(),
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

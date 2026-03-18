use crate::log_options::LogOptions;

pub struct WeiduLogOptions(Vec<LogOptions>);

impl WeiduLogOptions {
    pub fn new(options: Vec<LogOptions>) -> Self {
        Self(options)
    }
    pub fn to_args(&self, path: &str) -> Vec<String> {
        let mut out = vec![];
        if self.0.contains(&LogOptions::LogAppend) {
            out.push("--logapp".to_string());
        }
        for log in self.0.iter() {
            match log {
                LogOptions::LogAppend => {}
                _ => out.extend(log.to_args(path)),
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {

    use std::error::Error;

    use super::*;
    use pretty_assertions::assert_eq;
    #[test]
    fn test_split_log_mode() -> Result<(), Box<dyn Error>> {
        let tests = vec![
            (vec![LogOptions::Log("/".into())], vec!["--log", "/"]),
            (
                vec![LogOptions::Log("/".into()), LogOptions::LogAppend],
                vec!["--logapp", "--log", "/"],
            ),
            (
                vec![LogOptions::AutoLog, LogOptions::LogAppend],
                vec!["--logapp", "--autolog"],
            ),
        ];
        let empty_path = "".into();
        for (test, expected) in tests {
            assert_eq!(WeiduLogOptions::new(test).to_args(empty_path), expected)
        }
        Ok(())
    }
}

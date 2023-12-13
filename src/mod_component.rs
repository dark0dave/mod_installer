use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{PathBuf, MAIN_SEPARATOR},
};

#[derive(Debug, PartialEq, Clone)]
pub struct ModComponent {
    pub tp_file: String,
    pub name: String,
    pub lang: String,
    pub component: String,
}

impl From<String> for ModComponent {
    fn from(line: String) -> Self {
        let mut parts = line.split('~');

        let install_path = parts
            .nth(1)
            .unwrap_or_else(|| panic!("Could not get full name of mod, from: {}", line))
            .to_string();

        let tp_file = install_path
            .split(MAIN_SEPARATOR)
            .nth(1)
            .unwrap_or_else(|| panic!("Could not find tp2 file, from: {}", line))
            .to_string();

        let name = install_path
            .split(MAIN_SEPARATOR)
            .next()
            .unwrap_or_else(|| panic!("Could not split {} into mod into name and component", line))
            .to_ascii_lowercase();

        let mut lang_and_component = parts
            .next()
            .unwrap_or_else(|| panic!("Could not find lang and component, from {}", line))
            .split(' ');

        let lang = lang_and_component
            .nth(1)
            .unwrap_or_else(|| panic!("Could not find lang, from: {}", line))
            .replace('#', "");

        let component = lang_and_component
            .next()
            .unwrap_or_else(|| panic!("Could not find component, from {}", line))
            .replace('#', "");

        ModComponent {
            tp_file,
            name,
            lang,
            component,
        }
    }
}

pub fn parse_weidu_log(weidu_log_path: PathBuf) -> Vec<ModComponent> {
    let file = File::open(weidu_log_path).expect("Could not open weidu log exiting");
    let reader = BufReader::new(file);

    reader
        .lines()
        .flat_map(|line| match line {
            // Ignore comments and empty lines
            Ok(component)
                if !component.is_empty()
                    && !component.starts_with('\n')
                    && !component.starts_with("//") =>
            {
                Some(ModComponent::from(component))
            }
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {

    use std::path::Path;

    use super::*;
    #[test]
    fn test_parse_weidu_log() {
        let test_log = Path::new("fixtures/test.log");
        let logs = parse_weidu_log(test_log.to_path_buf());
        assert_eq!(
            logs.first(),
            Some(&ModComponent {
                tp_file: "TEST.TP2".to_string(),
                name: "test_mod_name_1".to_string(),
                lang: "0".to_string(),
                component: "0".to_string()
            })
        );
        assert_eq!(
            logs.last(),
            Some(&ModComponent {
                tp_file: "TEST.TP2".to_string(),
                name: "test_mod_name_2".to_string(),
                lang: "0".to_string(),
                component: "0".to_string()
            })
        );
    }
}

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

#[derive(Debug, PartialEq)]
pub struct Mod {
    pub install_path: String,
    pub name: String,
    pub lang: String,
    pub component: String,
}

impl From<String> for Mod {
    fn from(line: String) -> Self {
        let mut parts = line.split('~');

        let install_path = parts
            .nth(1)
            .expect("Could not get full name of mod")
            .to_string();
        let name = install_path
            .split('/')
            .next()
            .expect("Could not split mod into name and component")
            .to_ascii_lowercase();

        let mut lang_and_component = parts
            .next()
            .expect("Could not find lang and component")
            .split(' ');

        let lang = lang_and_component
            .nth(1)
            .expect("Could not find lang")
            .replace('#', "");
        let component = lang_and_component
            .next()
            .expect("Could not find component")
            .replace('#', "");

        Mod {
            install_path,
            name,
            lang,
            component,
        }
    }
}

pub fn parse_weidu_log(weidu_log_path: PathBuf) -> Vec<Mod> {
    let file = File::open(weidu_log_path).expect("Could not open weidu log exiting");
    let reader = BufReader::new(file);

    reader
        .lines()
        .flat_map(|line| match line {
            // Ignore comments
            Ok(r#mod) if !r#mod.starts_with("//") => Some(Mod::from(r#mod)),
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
            Some(&Mod {
                install_path: "TEST_MOD_NAME_1/TEST.TP2".to_string(),
                name: "test_mod_name_1".to_string(),
                lang: "0".to_string(),
                component: "0".to_string()
            })
        );
        assert_eq!(
            logs.last(),
            Some(&Mod {
                install_path: "TEST_MOD_NAME_2/TEST.TP2".to_string(),
                name: "test_mod_name_2".to_string(),
                lang: "0".to_string(),
                component: "0".to_string()
            })
        );
    }
}

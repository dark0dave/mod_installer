use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct ModComponent {
    pub tp_file: String,
    pub name: String,
    pub lang: String,
    pub component: String,
    pub component_name: String,
    pub sub_component: String,
    pub version: String,
}

impl From<String> for ModComponent {
    fn from(line: String) -> Self {
        let mut parts = line.split('~');

        let install_path = parts
            .nth(1)
            .unwrap_or_else(|| {
                panic!(
                    "Could not get full name of mod, from provided string: {}",
                    line
                )
            })
            .to_string();

        // This allows for both linux, macos and windows parsing
        let (tp_file, name) = if let Some(windows_path) = install_path.split('\\').nth(1) {
            let name = install_path
                .split('\\')
                .next()
                .unwrap_or_else(|| {
                    panic!("Could not split {} into mod into name and component", line)
                })
                .to_ascii_lowercase();
            (windows_path.to_string(), name)
        } else if let Some(linux_path) = install_path.split('/').nth(1) {
            let name = install_path
                .split('/')
                .next()
                .unwrap_or_else(|| {
                    panic!("Could not split {} into mod into name and component", line)
                })
                .to_ascii_lowercase();
            (linux_path.to_string(), name)
        } else {
            panic!(
                "Could not find tp2 file name, from provided string: {}",
                line
            )
        };

        let mut tail = parts
            .next()
            .unwrap_or_else(|| {
                panic!(
                    "Could not find lang and component, from provided string {}",
                    line
                )
            })
            .split("//");

        let mut lang_and_component = tail.next().unwrap_or_default().split(' ');

        let lang = lang_and_component
            .nth(1)
            .unwrap_or_else(|| panic!("Could not find lang, from provided string: {}", line))
            .replace('#', "");

        let component = lang_and_component
            .next()
            .unwrap_or_else(|| panic!("Could not find component, from provided string {}", line))
            .replace('#', "");

        let mut component_name_sub_component_version = tail.next().unwrap_or_default().split(':');

        let mut component_name_sub_component = component_name_sub_component_version
            .next()
            .unwrap_or_default()
            .split("->");

        let component_name = component_name_sub_component
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();

        let sub_component = component_name_sub_component
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();

        let version = component_name_sub_component_version
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();

        ModComponent {
            tp_file,
            name,
            lang,
            component,
            component_name,
            sub_component,
            version,
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
    use super::*;
    use pretty_assertions::assert_eq;
    use std::path::Path;

    #[test]
    fn test_parse_weidu_log() {
        let test_log = Path::new("fixtures/test.log");
        let logs = parse_weidu_log(test_log.to_path_buf());
        assert_eq!(
            logs,
            vec![
                ModComponent {
                    tp_file: "TEST.TP2".to_string(),
                    name: "test_mod_name_1".to_string(),
                    lang: "0".to_string(),
                    component: "0".to_string(),
                    component_name: "test mod one".to_string(),
                    sub_component: "".to_string(),
                    version: "".to_string()
                },
                ModComponent {
                    tp_file: "TEST.TP2".to_string(),
                    name: "test_mod_name_1".to_string(),
                    lang: "0".to_string(),
                    component: "1".to_string(),
                    component_name: "test mod two".to_string(),
                    sub_component: "".to_string(),
                    version: "".to_string()
                },
                ModComponent {
                    tp_file: "END.TP2".to_string(),
                    name: "test_mod_name_2".to_string(),
                    lang: "0".to_string(),
                    component: "0".to_string(),
                    component_name: "test mod with subcomponent information".to_string(),
                    sub_component: "Standard installation".to_string(),
                    version: "".to_string()
                },
                ModComponent {
                    tp_file: "END.TP2".to_string(),
                    name: "test_mod_name_3".to_string(),
                    lang: "0".to_string(),
                    component: "0".to_string(),
                    component_name: "test mod with version".to_string(),
                    sub_component: "".to_string(),
                    version: "1.02".to_string()
                },
                ModComponent {
                    tp_file: "TWEAKS.TP2".to_string(),
                    name: "test_mod_name_4".to_string(),
                    lang: "0".to_string(),
                    component: "3346".to_string(),
                    component_name: "test mod with both subcomponent information and version"
                        .to_string(),
                    sub_component: "Casting speed only".to_string(),
                    version: "v16".to_string()
                }
            ]
        );
    }

    #[test]
    fn test_parse_windows() {
        let mod_string = r"~TOBEX\TOBEX.TP2~ #0 #100 // TobEx - Core: v28";
        let mod_component = ModComponent::from(mod_string.to_string());
        let expected = ModComponent {
            tp_file: "TOBEX.TP2".to_string(),
            name: "tobex".to_string(),
            lang: "0".to_string(),
            component: "100".to_string(),
            component_name: "TobEx - Core".to_string(),
            sub_component: "".to_string(),
            version: "v28".to_string(),
        };
        assert_eq!(mod_component, expected)
    }
}

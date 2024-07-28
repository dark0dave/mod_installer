// This should mirror the weidu component
// https://github.com/WeiDUorg/weidu/blob/devel/src/tp.ml#L98
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub(crate) struct Component {
    pub(crate) tp_file: String,
    pub(crate) name: String,
    pub(crate) lang: String,
    pub(crate) component: String,
    pub(crate) component_name: String,
    pub(crate) sub_component: String,
    pub(crate) version: String,
}

impl From<String> for Component {
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

        Component {
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_windows() {
        let mod_string = r"~TOBEX\TOBEX.TP2~ #0 #100 // TobEx - Core: v28";
        let mod_component = Component::from(mod_string.to_string());
        let expected = Component {
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

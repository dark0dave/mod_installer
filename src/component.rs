use std::error::Error;

// This should mirror the weidu component
// https://github.com/WeiDUorg/weidu/blob/devel/src/tp.ml#L98
#[derive(Debug, PartialOrd, Clone)]
pub(crate) struct Component {
    pub(crate) tp_file: String,
    pub(crate) name: String,
    pub(crate) lang: String,
    pub(crate) component: String,
    pub(crate) component_name: String,
    pub(crate) sub_component: String,
    pub(crate) version: String,
}

impl PartialEq for Component {
    fn eq(&self, other: &Self) -> bool {
        self.tp_file == other.tp_file
            && self.name.to_lowercase() == other.name.to_lowercase()
            && self.lang.to_lowercase() == other.lang.to_lowercase()
            && self.component.to_lowercase() == other.component.to_lowercase()
    }
}

impl Component {
    pub(crate) fn strict_matching(&self, other: &Self) -> bool {
        self.eq(other)
            && self.component_name == other.component_name
            && self.sub_component == other.sub_component
            && self.version == other.version
    }
}

impl TryFrom<String> for Component {
    type Error = Box<dyn Error>;

    fn try_from(line: String) -> Result<Self, Self::Error> {
        let mut parts = line.split('~');

        let install_path = parts.nth(1).ok_or(format!(
            "Could not get full name of mod, from provided string: {line}"
        ))?;

        let (tp_file, name) = if install_path.split('\\').nth(1).is_some() {
            let mut component_path_string = install_path
                .split('\\')
                .collect::<Vec<&str>>()
                .into_iter()
                .rev();
            (
                component_path_string.next().unwrap_or_default(),
                component_path_string.next().unwrap_or_default(),
            )
        } else if install_path.split('/').nth(1).is_some() {
            let mut component_path_string = install_path
                .split('/')
                .collect::<Vec<&str>>()
                .into_iter()
                .rev();
            (
                component_path_string.next().unwrap_or_default(),
                component_path_string.next().unwrap_or_default(),
            )
        } else {
            return Err(
                format!("Could not find tp2 file name, from provided string: {line}").into(),
            );
        };

        let mut tail = parts
            .next()
            .ok_or(format!(
                "Could not find lang and component, from provided string {line}"
            ))?
            .split("//");

        let mut lang_and_component = tail.next().unwrap_or_default().split(' ');

        let lang = lang_and_component
            .nth(1)
            .ok_or(format!("Could not find lang, from provided string: {line}"))?
            .replace('#', "");

        let component = lang_and_component
            .next()
            .ok_or(format!(
                "Could not find component, from provided string {line}"
            ))?
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

        Ok(Self {
            tp_file: tp_file.to_string(),
            name: name.to_string(),
            lang,
            component,
            component_name,
            sub_component,
            version,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_windows() -> Result<(), Box<dyn Error>> {
        let mod_string = r"~TOBEX\TOBEX.TP2~ #0 #100 // TobEx - Core: v28";
        let mod_component = Component::try_from(mod_string.to_string())?;
        let expected = Component {
            tp_file: "TOBEX.TP2".to_string(),
            name: "tobex".to_string(),
            lang: "0".to_string(),
            component: "100".to_string(),
            component_name: "TobEx - Core".to_string(),
            sub_component: "".to_string(),
            version: "v28".to_string(),
        };
        assert_eq!(mod_component, expected);
        Ok(())
    }

    #[test]
    fn test_strict_match() -> Result<(), Box<dyn Error>> {
        let non_strict_match_1 = Component {
            tp_file: "TOBEX.TP2".to_string(),
            name: "tobex".to_string(),
            lang: "0".to_string(),
            component: "100".to_string(),
            component_name: "TobEx - Core".to_string(),
            sub_component: "".to_string(),
            version: "v28".to_string(),
        };

        let non_strict_match_2 = Component {
            tp_file: "TOBEX.TP2".to_string(),
            name: "tobex".to_string(),
            lang: "0".to_string(),
            component: "100".to_string(),
            component_name: "TobEx - Core Chicken".to_string(),
            sub_component: "".to_string(),
            version: "v28".to_string(),
        };
        assert_eq!(non_strict_match_1, non_strict_match_2);
        assert_eq!(
            non_strict_match_1.strict_matching(&non_strict_match_2),
            false
        );
        Ok(())
    }
}

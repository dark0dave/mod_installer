use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    slice::Iter,
};

use crate::component::Component;

#[derive(Debug, PartialEq, PartialOrd)]
pub(crate) struct LogFile(pub(crate) Vec<Component>);

impl LogFile {
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Component) -> bool,
    {
        self.0.retain_mut(|elem| f(elem));
    }
}

impl<'a> IntoIterator for &'a LogFile {
    type Item = &'a Component;
    type IntoIter = Iter<'a, Component>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl TryFrom<PathBuf> for LogFile {
    type Error = Box<dyn Error>;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let file = File::open(value)?;
        let reader = BufReader::new(file);
        let mut components = vec![];

        for line in reader.lines().map_while(|line| line.ok()) {
            // Ignore comments and empty lines
            if !line.is_empty() && !line.starts_with('\n') && !line.starts_with("//") {
                components.push(Component::try_from(line)?)
            }
        }
        Ok(Self(components))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn test_parse_weidu_log() -> Result<(), Box<dyn Error>> {
        let test_log = PathBuf::from("fixtures/test.log");
        let result = LogFile::try_from(test_log)?;
        let expected = LogFile(vec![
            Component {
                tp_file: "TEST.TP2".to_string(),
                name: "test_mod_name_1".to_string(),
                lang: "0".to_string(),
                component: "0".to_string(),
                component_name: "test mod one".to_string(),
                sub_component: "".to_string(),
                version: "".to_string(),
            },
            Component {
                tp_file: "TEST.TP2".to_string(),
                name: "test_mod_name_1".to_string(),
                lang: "0".to_string(),
                component: "1".to_string(),
                component_name: "test mod two".to_string(),
                sub_component: "".to_string(),
                version: "".to_string(),
            },
            Component {
                tp_file: "END.TP2".to_string(),
                name: "test_mod_name_2".to_string(),
                lang: "0".to_string(),
                component: "0".to_string(),
                component_name: "test mod with subcomponent information".to_string(),
                sub_component: "Standard installation".to_string(),
                version: "".to_string(),
            },
            Component {
                tp_file: "END.TP2".to_string(),
                name: "test_mod_name_3".to_string(),
                lang: "0".to_string(),
                component: "0".to_string(),
                component_name: "test mod with version".to_string(),
                sub_component: "".to_string(),
                version: "1.02".to_string(),
            },
            Component {
                tp_file: "TWEAKS.TP2".to_string(),
                name: "test_mod_name_4".to_string(),
                lang: "0".to_string(),
                component: "3346".to_string(),
                component_name: "test mod with both subcomponent information and version"
                    .to_string(),
                sub_component: "Casting speed only".to_string(),
                version: "v16".to_string(),
            },
        ]);
        assert_eq!(expected, result);
        Ok(())
    }
}

use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    slice::Iter,
};

use config::{log_options::LogOptions, weidu_log_options::WeiduLogOptions};

use crate::{
    runner::LINE_ENDING, weidu_component::WeiduComponent, weidu_install_block::WeiduInstallBlock,
};

#[derive(Debug, PartialEq, PartialOrd)]
pub(crate) struct WeiduBatchedComponents(Vec<WeiduComponent>);

impl WeiduBatchedComponents {
    pub(crate) fn remove_existing(
        &mut self,
        strict_matching: bool,
        game_directory: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let number_of_mods_found = self.len();
        let existing_weidu_log_file_path = game_directory.join("weidu").with_extension("log");
        if let Ok(installed_mods) = WeiduBatchedComponents::try_from(existing_weidu_log_file_path) {
            for installed_mod in &installed_mods {
                if strict_matching {
                    self.retain(|mod_to_install| installed_mod.strict_matching(mod_to_install));
                } else {
                    self.retain(|mod_to_install| installed_mod != mod_to_install);
                }
            }
        }

        log::info!(
            "Number of mods found: {}, Number of mods to be installed: {}",
            number_of_mods_found,
            self.len()
        );
        Ok(())
    }
    pub(crate) fn push(&mut self, component: WeiduComponent) {
        self.0.push(component);
    }
    pub(crate) fn last(&self) -> Option<&WeiduComponent> {
        self.0.last()
    }
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
    pub(crate) fn retain<F: FnMut(&WeiduComponent) -> bool>(&mut self, mut f: F) {
        self.0.retain_mut(|elem| f(elem));
    }
}

impl<'a> IntoIterator for &'a WeiduBatchedComponents {
    type Item = &'a WeiduComponent;
    type IntoIter = Iter<'a, WeiduComponent>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl TryFrom<PathBuf> for WeiduBatchedComponents {
    type Error = Box<dyn Error>;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let file = File::open(value)?;
        let reader = BufReader::new(file);
        let mut components = vec![];

        for line in reader.lines().map_while(|line| line.ok()) {
            // Ignore comments and empty lines
            if !line.is_empty() && !line.starts_with(LINE_ENDING) && !line.starts_with("//") {
                components.push(WeiduComponent::try_from(line)?)
            }
        }
        Ok(Self(components))
    }
}

impl From<Vec<WeiduComponent>> for WeiduBatchedComponents {
    fn from(components: Vec<WeiduComponent>) -> Self {
        Self(components)
    }
}

impl WeiduInstallBlock for WeiduBatchedComponents {
    fn generate_weidu_args(
        &self,
        weidu_log_mode: Vec<LogOptions>,
        language: &str,
        generic_weidu_args: &[String],
    ) -> Vec<String> {
        let mut args = vec![];
        for (position, component) in self.0.iter().enumerate() {
            if position == 0 {
                args.push(component.full_component_name());
                args.push("--force-install-list".into());
            }
            args.push(component.component.to_string());
            if position == self.0.len() - 1 {
                args.push("--use-lang".to_string());
                args.push(language.to_string());
                args.push("--language".to_string());
                args.push(component.lang.to_string());
                args.push("--no-exit-pause".to_string());
            }
        }

        args.extend(WeiduLogOptions::new(weidu_log_mode).to_args(&self.log_file_name()));
        args.extend_from_slice(generic_weidu_args);
        args
    }
    fn log_file_name(&self) -> String {
        let mut name = String::new();
        for (position, component) in self.0.iter().enumerate() {
            if position == 0 {
                name += &component.name.to_lowercase();
            }
            name += "-";
            name += component.component.as_str();
            if position == self.0.len() - 1 {
                name += ".log"
            }
        }
        name
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
        let result = WeiduBatchedComponents::try_from(test_log)?;
        let expected = WeiduBatchedComponents(vec![
            WeiduComponent {
                tp_file: "TEST.TP2".to_string(),
                name: "test_mod_name_1".to_string(),
                lang: "0".to_string(),
                component: "0".to_string(),
                component_name: "test mod one".to_string(),
                sub_component: "".to_string(),
                version: "".to_string(),
            },
            WeiduComponent {
                tp_file: "TEST.TP2".to_string(),
                name: "test_mod_name_1".to_string(),
                lang: "0".to_string(),
                component: "1".to_string(),
                component_name: "test mod two".to_string(),
                sub_component: "".to_string(),
                version: "".to_string(),
            },
            WeiduComponent {
                tp_file: "END.TP2".to_string(),
                name: "test_mod_name_2".to_string(),
                lang: "0".to_string(),
                component: "0".to_string(),
                component_name: "test mod with subcomponent information".to_string(),
                sub_component: "Standard installation".to_string(),
                version: "".to_string(),
            },
            WeiduComponent {
                tp_file: "END.TP2".to_string(),
                name: "test_mod_name_3".to_string(),
                lang: "0".to_string(),
                component: "0".to_string(),
                component_name: "test mod with version".to_string(),
                sub_component: "".to_string(),
                version: "1.02".to_string(),
            },
            WeiduComponent {
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

    #[test]
    fn test_find_mods_skip_installed() -> Result<(), Box<dyn Error>> {
        let mut log_file = WeiduBatchedComponents::try_from(PathBuf::from("./fixtures/test.log"))?;
        let game_directory = PathBuf::from("./fixtures");
        log_file.remove_existing(false, &game_directory)?;
        let expected = WeiduBatchedComponents::try_from(PathBuf::from("./fixtures/expected.log"))?;
        assert_eq!(expected, log_file);
        Ok(())
    }
}

use crate::state::State;

use serde_derive::{Deserialize, Serialize};

pub(crate) const PARSER_CONFIG_LOCATION: &str = "parser";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct ParserConfig {
    pub(crate) in_progress_words: Vec<String>,
    pub(crate) useful_status_words: Vec<String>,
    pub(crate) choice_words: Vec<String>,
    pub(crate) choice_phrase: Vec<String>,
    pub(crate) completed_with_warnings: Vec<String>,
    pub(crate) failed_with_error: Vec<String>,
    pub(crate) finished: Vec<String>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            in_progress_words: vec!["installing".to_string(), "creating".to_string()],
            useful_status_words: vec![
                "copied".to_string(),
                "copying".to_string(),
                "creating".to_string(),
                "installed".to_string(),
                "installing".to_string(),
                "patched".to_string(),
                "patching".to_string(),
                "processed".to_string(),
                "processing".to_string(),
            ],
            choice_words: vec![
                "choice".to_string(),
                "choose".to_string(),
                "select".to_string(),
                "enter".to_string(),
            ],
            choice_phrase: vec![
                "do you want".to_string(),
                "would you like".to_string(),
                "answer [y]es or [n]o.".to_string(),
                "is this correct? [y]es or [n]o".to_string(),
            ],
            completed_with_warnings: vec!["installed with warnings".to_string()],
            failed_with_error: vec![
                "not installed due to errors".to_string(),
                "installation aborted".to_string(),
            ],
            finished: vec![
                "successfully installed".to_string(),
                "process ended".to_string(),
            ],
        }
    }
}

impl ParserConfig {
    pub fn string_looks_like_question(&self, weidu_output: &str) -> bool {
        let comparable_output = weidu_output.trim().to_ascii_lowercase();
        // installing|creating
        for progress_word in self.in_progress_words.iter() {
            if comparable_output.contains(progress_word) {
                return false;
            }
        }

        for question in self.choice_phrase.iter() {
            if comparable_output.contains(question) {
                return true;
            }
        }

        for question in self.choice_words.iter() {
            for word in comparable_output.split_whitespace() {
                if word
                    .chars()
                    .filter(|c| c.is_alphabetic())
                    .collect::<String>()
                    == *question
                {
                    return true;
                }
            }
        }

        false
    }

    pub fn detect_weidu_finished_state(&self, weidu_output: &str) -> State {
        let comparable_output = weidu_output.trim().to_lowercase();
        let failure = self.failed_with_error.iter().fold(false, |acc, fail_case| {
            comparable_output.contains(fail_case) || acc
        });
        if failure {
            return State::CompletedWithErrors {
                error_details: comparable_output,
            };
        }
        let warning = self
            .completed_with_warnings
            .iter()
            .fold(false, |acc, warn_case| {
                comparable_output.contains(warn_case) || acc
            });
        if warning {
            return State::CompletedWithWarnings;
        }
        let finished = self.finished.iter().fold(false, |acc, success_case| {
            comparable_output.contains(success_case) || acc
        });
        if finished {
            return State::Completed;
        }
        State::InProgress
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;
    use std::{error::Error, path::Path, result::Result};

    #[test]
    fn test_exit_warnings() -> Result<(), Box<dyn Error>> {
        let config = ParserConfig::default();
        let test = "INSTALLED WITH WARNINGS     Additional equipment for Thieves and Bards";
        assert_eq!(config.string_looks_like_question(test), false);
        assert_eq!(
            config.detect_weidu_finished_state(test),
            State::CompletedWithWarnings
        );
        Ok(())
    }

    #[test]
    fn test_exit_success() -> Result<(), Box<dyn Error>> {
        let config = ParserConfig::default();
        let test = "SUCCESSFULLY INSTALLED      Jan's Extended Quest";
        assert_eq!(config.string_looks_like_question(test), false);
        assert_eq!(config.detect_weidu_finished_state(test), State::Completed);
        Ok(())
    }

    #[test]
    fn is_a_question() -> Result<(), Box<dyn Error>> {
        let config = ParserConfig::default();
        let tests = vec![
            "Enter the full path to your Baldur's Gate installation then press Enter.",
            "Enter the full path to your BG:EE+SoD installation then press Enter.\
Example: C:\\Program Files (x86)\\BeamDog\\Games\\00806",
            "[N]o, [Q]uit or choose one:",
            "Please enter the chance for items to randomly not be randomised as a integet number (e.g. 10 for 10%)",
            "Is this correct? [Y]es or [N]o",
        ];
        for test in tests {
            assert_eq!(
                config.string_looks_like_question(test),
                true,
                "String {} doesn't look like a question",
                test
            );
            assert_eq!(config.detect_weidu_finished_state(test), State::InProgress);
        }
        Ok(())
    }

    #[test]
    fn is_not_a_question() -> Result<(), Box<dyn Error>> {
        let config = ParserConfig::default();
        let tests = vec![
            "FAILURE:",
            "NOT INSTALLED DUE TO ERRORS The BG1 NPC Project: Required Modifications",
            "Creating epilogues. Too many epilogues... Why are there so many options here?",
            "Including file(s) spellchoices_defensive/vanilla/ENCHANTER.TPH",
        ];
        for test in tests {
            assert_eq!(
                config.string_looks_like_question(test),
                false,
                "String {} does look like a question",
                test
            );
        }
        Ok(())
    }

    #[test]
    fn load_config() -> Result<(), Box<dyn Error>> {
        let root = std::env::current_dir()?;
        let config_path = Path::join(&root, Path::new("example_config.toml"));
        let config: ParserConfig = confy::load_path(config_path)?;
        let expected = ParserConfig::default();
        assert_eq!(expected, config);
        Ok(())
    }

    #[test]
    fn failure() -> Result<(), Box<dyn Error>> {
        let config = ParserConfig::default();
        let tests = vec![
            "not installed due to errors the bg1 npc project: required modifications",
            "installation aborted merge dlc into game -> merge all available dlcs",
        ];
        for input in tests {
            assert_eq!(
                config.detect_weidu_finished_state(input),
                State::CompletedWithErrors {
                    error_details: input.to_string(),
                },
                "Input {} did not fail",
                input
            );
        }
        Ok(())
    }
}

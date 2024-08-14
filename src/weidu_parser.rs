use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{Receiver, Sender, TryRecvError},
        Arc,
    },
    thread,
};

use crate::{state::State, utils::sleep};

const WEIDU_USEFUL_STATUS: [&str; 9] = [
    "copied",
    "copying",
    "creating",
    "installed",
    "installing",
    "patched",
    "patching",
    "processed",
    "processing",
];

const WEIDU_CHOICE_WORDS: [&str; 4] = ["choice", "choose", "select", "enter"];

const WEIDU_CHOICE_PHRASE: [&str; 2] = ["do you want", "would you like"];

const WEIDU_COMPLETED_WITH_WARNINGS: &str = "installed with warnings";

const WEIDU_FAILED_WITH_ERROR: &str = "not installed due to errors";

const WEIDU_FINISHED: &str = "successfully installed";

const EET_FINISHED: &str = "Process ended";

#[derive(Debug)]
enum ParserState {
    CollectingQuestion,
    WaitingForMoreQuestionContent,
    LookingForInterestingOutput,
}

pub(crate) fn parse_raw_output(
    sender: Sender<State>,
    receiver: Receiver<String>,
    wait_count: Arc<AtomicUsize>,
    timeout: usize,
) {
    let mut current_state = ParserState::LookingForInterestingOutput;
    let mut question = String::new();
    sender
        .send(State::InProgress)
        .expect("Failed to send process start event");
    thread::spawn(move || loop {
        match receiver.try_recv() {
            Ok(string) => match current_state {
                ParserState::CollectingQuestion | ParserState::WaitingForMoreQuestionContent => {
                    if WEIDU_USEFUL_STATUS.contains(&string.as_str()) {
                        log::debug!(
                            "Weidu seems to know an answer for the last question, ignoring it"
                        );
                        current_state = ParserState::LookingForInterestingOutput;
                        question.clear();
                    } else {
                        log::debug!("Appending line '{}' to user question", string);
                        question.push_str(&string);
                        current_state = ParserState::CollectingQuestion;
                    }
                }
                ParserState::LookingForInterestingOutput => {
                    log::trace!("{}", string);
                    if let Some(weidu_finished_state) = detect_weidu_finished_state(&string) {
                        sender
                            .send(weidu_finished_state)
                            .expect("Failed to send process error event");
                        break;
                    } else if string_looks_like_question(&string) {
                        log::debug!(
                            "Changing parser state to '{:?}' due to line {}",
                            ParserState::CollectingQuestion,
                            string
                        );
                        current_state = ParserState::CollectingQuestion;
                        question.push_str(string.as_str());
                    }
                    if !string.trim().is_empty() {
                        log::trace!("{}", string);
                    }
                }
            },
            Err(TryRecvError::Empty) => {
                match current_state {
                    ParserState::CollectingQuestion => {
                        log::debug!(
                            "Changing parser state to '{:?}'",
                            ParserState::WaitingForMoreQuestionContent
                        );
                        current_state = ParserState::WaitingForMoreQuestionContent;
                    }
                    ParserState::WaitingForMoreQuestionContent => {
                        log::debug!("No new weidu output, sending question to user");
                        sender
                            .send(State::RequiresInput { question })
                            .expect("Failed to send question");
                        current_state = ParserState::LookingForInterestingOutput;
                        question = String::new();
                    }
                    _ => {
                        if wait_count.load(Ordering::Relaxed) >= timeout {
                            sender
                                .send(State::TimedOut)
                                .expect("Could send timeout error");
                        }
                        // there is no new weidu output and we are not waiting for any, so there is nothing to do
                    }
                }
                sleep(100);
            }
            Err(TryRecvError::Disconnected) => {
                sender
                    .send(State::Completed)
                    .expect("Failed to send provess end event");
                break;
            }
        }
    });
}

fn string_looks_like_question(weidu_output: &str) -> bool {
    let comparable_output = weidu_output.trim().to_ascii_lowercase();
    // installing|creating
    if comparable_output.contains(WEIDU_USEFUL_STATUS[2])
        || comparable_output.contains(WEIDU_USEFUL_STATUS[4])
    {
        return false;
    }

    for question in WEIDU_CHOICE_PHRASE {
        if comparable_output.contains(question) {
            return true;
        }
    }

    for question in WEIDU_CHOICE_WORDS {
        for word in comparable_output.split_whitespace() {
            if word
                .chars()
                .filter(|c| c.is_alphabetic())
                .collect::<String>()
                == question
            {
                return true;
            }
        }
    }

    false
}

fn detect_weidu_finished_state(weidu_output: &str) -> Option<State> {
    let comparable_output = weidu_output.trim().to_lowercase();
    if comparable_output.contains(WEIDU_FAILED_WITH_ERROR) {
        Some(State::CompletedWithErrors {
            error_details: comparable_output,
        })
    } else if comparable_output.contains(WEIDU_COMPLETED_WITH_WARNINGS) {
        Some(State::CompletedWithWarnings)
    } else if comparable_output.contains(WEIDU_FINISHED) || comparable_output.contains(EET_FINISHED)
    {
        Some(State::Completed)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_exit_warnings() {
        let test = "INSTALLED WITH WARNINGS     Additional equipment for Thieves and Bards";
        assert_eq!(string_looks_like_question(test), false);
        assert_eq!(
            detect_weidu_finished_state(test),
            Some(State::CompletedWithWarnings)
        )
    }
    #[test]
    fn test_exit_success() {
        let test = "SUCCESSFULLY INSTALLED      Jan's Extended Quest";
        assert_eq!(string_looks_like_question(test), false);
        assert_eq!(detect_weidu_finished_state(test), Some(State::Completed))
    }

    #[test]
    fn is_not_question() {
        let test = "Creating epilogues. Too many epilogues... Why are there so many options here?";
        assert_eq!(string_looks_like_question(test), false);
        let test = "Including file(s) spellchoices_defensive/vanilla/ENCHANTER.TPH";
        assert_eq!(string_looks_like_question(test), false);
    }

    #[test]
    fn is_a_question() {
        let tests = vec!["Enter the full path to your Baldur's Gate installation then press Enter.", "Enter the full path to your BG:EE+SoD installation then press Enter.\
Example: C:\\Program Files (x86)\\BeamDog\\Games\\00806", "[N]o, [Q]uit or choose one:", "Please enter the chance for items to randomly not be randomised as a integet number (e.g. 10 for 10%)"];
        for question in tests {
            assert_eq!(
                string_looks_like_question(question),
                true,
                "String {} doesn't look like a question",
                question
            );
        }
    }

    #[test]
    fn is_not_a_question() {
        let tests = vec![
            "FAILURE:",
            "NOT INSTALLED DUE TO ERRORS The BG1 NPC Project: Required Modifications",
        ];
        for question in tests {
            assert_eq!(
                string_looks_like_question(question),
                false,
                "String {} does look like a question",
                question
            );
        }
    }
}

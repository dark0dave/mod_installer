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

const WEIDU_CHOICE: [&str; 6] = [
    "choice",
    "choose",
    "select",
    "do you want",
    "would you like",
    "enter",
];

const WEIDU_CHOICE_SYMBOL: [char; 2] = ['?', ':'];

const WEIDU_COMPLETED_WITH_WARNINGS: &str = "installed with warnings";

const WEIDU_FAILED_WITH_ERROR: &str = "not installed due to errors";

const WEIDU_FINISHED: &str = "successfully installed";

#[derive(Debug)]
enum ParserState {
    CollectingQuestion,
    WaitingForMoreQuestionContent,
    LookingForInterestingOutput,
}

pub fn parse_raw_output(
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
                    let may_be_weidu_finished_state = detect_weidu_finished_state(&string);
                    if let Some(weidu_finished_state) = may_be_weidu_finished_state {
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
    (WEIDU_CHOICE.contains(&comparable_output.as_str()))
        || WEIDU_CHOICE_SYMBOL.contains(&comparable_output.chars().last().unwrap_or_default())
        || comparable_output
            .split(' ')
            .take(1)
            .any(|c| WEIDU_CHOICE.contains(&c))
}

fn detect_weidu_finished_state(weidu_output: &str) -> Option<State> {
    let comparable_output = weidu_output.trim().to_lowercase();
    if comparable_output.contains(WEIDU_FAILED_WITH_ERROR) {
        Some(State::CompletedWithErrors {
            error_details: comparable_output,
        })
    } else if comparable_output.contains(WEIDU_COMPLETED_WITH_WARNINGS) {
        Some(State::CompletedWithWarnings)
    } else if comparable_output.contains(WEIDU_FINISHED) {
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
        assert_eq!(string_looks_like_question(test), false)
    }

    #[test]
    fn is_a_question() {
        let test = "Enter the full path to your Baldur's Gate installation then press Enter.";
        assert_eq!(string_looks_like_question(test), true);
        let test = "Enter the full path to your BG:EE+SoD installation then press Enter.\
Example: C:\\Program Files (x86)\\BeamDog\\Games\\00806";
        assert_eq!(string_looks_like_question(test), true)
    }
}

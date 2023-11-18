use std::{
    sync::mpsc::{Receiver, Sender, TryRecvError},
    thread,
};

use crate::{state::State, utils::sleep};

const WEIDU_USEFUL_STATUS: [&str; 8] = [
    "copying",
    "copied",
    "installing",
    "installed",
    "patching",
    "patched",
    "processing",
    "processed",
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

#[derive(Debug)]
enum ParserState {
    CollectingQuestion,
    WaitingForMoreQuestionContent,
    LookingForInterestingOutput,
}

pub fn parse_raw_output(sender: Sender<State>, receiver: Receiver<String>) {
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
                        log::debug!("No new weidu otput, sending question to user");
                        sender
                            .send(State::RequiresInput { question })
                            .expect("Failed to send question");
                        current_state = ParserState::LookingForInterestingOutput;
                        question = String::new();
                    }
                    _ => {
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
    let comparable_output = weidu_output.trim().to_lowercase();
    if comparable_output.contains("installing") {
        return false;
    }
    (WEIDU_CHOICE.contains(&comparable_output.as_str()))
        || WEIDU_CHOICE_SYMBOL.contains(
            &comparable_output
                .as_str()
                .chars()
                .last()
                .unwrap_or_default(),
        )
}

fn detect_weidu_finished_state(weidu_output: &str) -> Option<State> {
    let comparable_output = weidu_output.trim().to_lowercase();
    if WEIDU_FAILED_WITH_ERROR.eq(&comparable_output) {
        Some(State::CompletedWithErrors {
            error_details: comparable_output,
        })
    } else if WEIDU_COMPLETED_WITH_WARNINGS.eq(&comparable_output) {
        Some(State::CompletedWithWarnings)
    } else {
        None
    }
}

use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{Receiver, Sender, TryRecvError},
        Arc, RwLock,
    },
    thread,
};

use crate::{config::parser_config::ParserConfig, state::State, utils::sleep};

#[derive(Debug)]
enum ParserState {
    CollectingQuestion,
    WaitingForMoreQuestionContent,
    LookingForInterestingOutput,
}

pub(crate) fn parse_raw_output(
    sender: Sender<State>,
    receiver: Receiver<String>,
    parser_config: Arc<ParserConfig>,
    wait_count: Arc<AtomicUsize>,
    log: Arc<RwLock<String>>,
    timeout: usize,
    tick: u64,
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
                    if let Ok(mut writer) = log.write() {
                        writer.push_str(&string);
                    }
                    if parser_config.useful_status_words.contains(&string) {
                        log::debug!(
                            "Weidu seems to know an answer for the last question, ignoring it"
                        );
                        current_state = ParserState::LookingForInterestingOutput;
                        question.clear();
                    } else {
                        log::debug!("Appending line '{string}' to user question");
                        question.push_str(&string);
                        current_state = ParserState::CollectingQuestion;
                    }
                }
                ParserState::LookingForInterestingOutput => {
                    if let Ok(mut writer) = log.write() {
                        writer.push_str(&string);
                    }
                    let installer_state = parser_config.detect_weidu_finished_state(&string);
                    if installer_state != State::InProgress {
                        sender
                            .send(installer_state)
                            .expect("Failed to send process error event");
                        break;
                    } else if parser_config.string_looks_like_question(&string) {
                        log::debug!(
                            "Changing parser state to '{:?}' due to line {}",
                            ParserState::CollectingQuestion,
                            string
                        );
                        current_state = ParserState::CollectingQuestion;
                        question.push_str(string.as_str());
                    }
                    if !string.trim().is_empty() {
                        log::trace!("{string}");
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
                sleep(tick);
            }
            Err(TryRecvError::Disconnected) => {
                sender
                    .send(State::Completed)
                    .expect("Failed to send process end event");
                break;
            }
        }
    });
}

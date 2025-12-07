use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc::{Receiver, Sender, TryRecvError},
    },
    thread,
};

use crate::{config::parser_config::ParserConfig, state::State};

#[derive(Debug)]
enum ParserState {
    CollectingQuestion { question: String },
    WaitingForMoreQuestionContent { question: String },
    LookingForInterestingOutput,
}

pub(crate) fn parse_raw_output(
    sender: Sender<State>,
    receiver: Receiver<String>,
    parser_config: Arc<ParserConfig>,
    wait_count: Arc<AtomicUsize>,
    timeout: usize,
) {
    let mut current_state = ParserState::LookingForInterestingOutput;
    sender
        .send(State::InProgress)
        .expect("Failed to send process start event");
    thread::spawn(move || {
        loop {
            let mut weidu_log = String::new();
            match receiver.try_recv() {
                Ok(string) => match current_state {
                    ParserState::CollectingQuestion { mut question }
                    | ParserState::WaitingForMoreQuestionContent { mut question } => {
                        if parser_config.useful_status_words.contains(&string) {
                            log::debug!(
                                "Weidu seems to know an answer for the last question, ignoring it"
                            );
                            current_state = ParserState::LookingForInterestingOutput;
                        } else {
                            log::debug!("Appending line '{string}' to user question");
                            question.push_str(&string);
                            current_state = ParserState::CollectingQuestion { question };
                        }
                    }
                    ParserState::LookingForInterestingOutput => {
                        weidu_log.push_str(&string);
                        let installer_state = parser_config.detect_weidu_finished_state(weidu_log);
                        if installer_state != State::InProgress {
                            sender
                                .send(installer_state)
                                .expect("Failed to send process error event");
                            break;
                        }
                        if parser_config.string_looks_like_question(&string) {
                            current_state = ParserState::CollectingQuestion {
                                question: string.clone(),
                            };
                            log::debug!("Changed parser state to '{:?}'", current_state);
                        }
                        if !string.trim().is_empty() {
                            log::trace!("{string}");
                        }
                    }
                },
                Err(TryRecvError::Empty) => match current_state {
                    ParserState::CollectingQuestion { question } => {
                        current_state = ParserState::WaitingForMoreQuestionContent { question };
                        log::debug!("Changed parser state to '{:?}'", current_state,);
                    }
                    ParserState::WaitingForMoreQuestionContent { question } => {
                        log::debug!("No new weidu output, sending question to user");
                        sender
                            .send(State::RequiresInput { question })
                            .expect("Failed to send question");
                        current_state = ParserState::LookingForInterestingOutput;
                    }
                    _ if wait_count.load(Ordering::Relaxed) >= timeout => {
                        sender
                            .send(State::TimedOut { timeout, weidu_log })
                            .expect("Could send timeout error");
                    }
                    _ => {}
                },
                Err(TryRecvError::Disconnected) => {
                    sender
                        .send(State::Completed)
                        .expect("Failed to send process end event");
                    break;
                }
            }
        }
    });
}

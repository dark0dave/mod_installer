use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc::{Receiver, Sender, TryRecvError},
    },
    thread,
};

use config::{parser_config::ParserConfig, state::State};

use crate::utils::sleep;

#[derive(Debug)]
enum ParserState {
    CollectingQuestion,
    WaitingForMoreQuestionContent,
    LookingForInterestingOutput,
}

pub(crate) fn parse_raw_output(
    tick: u64,
    sender: Sender<State>,
    receiver: Receiver<String>,
    parser_config: Arc<ParserConfig>,
    wait_count: Arc<AtomicUsize>,
    timeout: usize,
) {
    let mut current_state = ParserState::LookingForInterestingOutput;
    let mut buffer = vec![];
    let mut question = vec![];
    let mut grace_ticks: usize = 3;
    sender
        .send(State::InProgress)
        .expect("Failed to send process start event");
    thread::spawn(move || {
        loop {
            match receiver.try_recv() {
                Ok(string) => {
                    log::info!("{string}");
                    let installer_state = parser_config.detect_weidu_finished_state(&string);
                    if installer_state != State::InProgress {
                        sender
                            .send(installer_state)
                            .expect("Failed to send process error event");
                        break;
                    }
                    buffer.push(string.clone());
                    match current_state {
                        ParserState::CollectingQuestion
                        | ParserState::WaitingForMoreQuestionContent => {
                            if parser_config.useful_status_words.contains(&string) {
                                log::debug!(
                                    "Weidu seems to know an answer for the last question, ignoring it"
                                );
                                current_state = ParserState::LookingForInterestingOutput;
                                question.clear();
                            } else {
                                log::debug!("Appending line '{string}' to user question");
                                question.push(string);
                                current_state = ParserState::CollectingQuestion;
                            }
                        }
                        ParserState::LookingForInterestingOutput => {
                            if parser_config.string_looks_like_question(&string) {
                                let lower = string.to_ascii_lowercase();
                                if lower.contains("[a]ccept")
                                    || lower.contains("[r]etry")
                                    || lower.contains("[c]ancel")
                                {
                                    question.push(string.clone());
                                    sender
                                        .send(State::RequiresInput {
                                            question: question.join(""),
                                        })
                                        .expect("Failed to send question");
                                    current_state = ParserState::LookingForInterestingOutput;
                                    question.clear();
                                    continue;
                                }
                                log::debug!(
                                    "Changing parser state to '{:?}' due to line {}",
                                    ParserState::CollectingQuestion,
                                    string
                                );
                                current_state = ParserState::CollectingQuestion;
                                let keep = 2000usize;
                                let start = buffer.len().saturating_sub(keep);
                                for history in buffer.iter().skip(start) {
                                    question.push(history.clone());
                                }
                            }
                        }
                    }
                }
                Err(TryRecvError::Empty) => match current_state {
                    ParserState::CollectingQuestion if grace_ticks > 0 => {
                        log::debug!("Collecting question, with grace of {grace_ticks} remaining");
                        sleep(tick);
                        grace_ticks -= 1;
                    }
                    ParserState::CollectingQuestion => {
                        log::debug!(
                            "Changing parser state to '{:?}'",
                            ParserState::WaitingForMoreQuestionContent
                        );
                        current_state = ParserState::WaitingForMoreQuestionContent;
                        grace_ticks = 3;
                    }
                    ParserState::WaitingForMoreQuestionContent => {
                        log::debug!("No new weidu output, sending question to user");
                        sender
                            .send(State::RequiresInput {
                                question: question.join(""),
                            })
                            .expect("Failed to send question");
                        current_state = ParserState::LookingForInterestingOutput;
                        question.clear();
                    }
                    _ if wait_count.load(Ordering::Relaxed) >= timeout => {
                        sender
                            .send(State::TimedOut)
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

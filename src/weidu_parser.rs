use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc::{Receiver, Sender, TryRecvError},
    },
    thread,
};

use config::{args::Options, parser_config::ParserConfig, state::State};

use crate::utils::sleep;

#[derive(Debug)]
enum ParserState {
    CollectingQuestion,
    WaitingForMoreQuestionContent,
    LookingForInterestingOutput,
}

pub(crate) fn parse_raw_output(
    options: &Options,
    sender: Sender<State>,
    receiver: Receiver<String>,
    parser_config: Arc<ParserConfig>,
    wait_count: Arc<AtomicUsize>,
) {
    let mut current_state = ParserState::LookingForInterestingOutput;
    let mut buffer = vec![];
    let mut question = vec![];
    let mut grace_ticks: usize = 3;
    if let Err(err) = sender.send(State::InProgress) {
        log::error!("Failed to send process start event, {err}");
        return;
    }
    let options = options.clone();
    thread::spawn(move || {
        loop {
            match receiver.try_recv() {
                Ok(string) => {
                    log::info!("{string}");
                    let installer_state = parser_config.detect_weidu_finished_state(&string);
                    if installer_state != State::InProgress
                        && let Err(err) = sender.send(installer_state)
                    {
                        log::error!("Failed to send process error event. {err}");
                        return;
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
                                log::debug!(
                                    "Changing parser state to '{:?}' due to line {}",
                                    ParserState::CollectingQuestion,
                                    string
                                );
                                current_state = ParserState::CollectingQuestion;
                                let min_index = buffer.len().saturating_sub(options.lookback);
                                for history in buffer.get(min_index..).unwrap_or_default() {
                                    question.push(history.clone());
                                }
                            }
                        }
                    }
                }
                Err(TryRecvError::Empty) => match current_state {
                    ParserState::CollectingQuestion if grace_ticks > 0 => {
                        log::debug!("Collecting question, with grace of {grace_ticks} remaining");
                        sleep(options.tick);
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
                        if let Err(err) = sender.send(State::RequiresInput {
                            question: question.join(""),
                        }) {
                            log::error!("Failed to send question: {err}");
                            return;
                        }
                        current_state = ParserState::LookingForInterestingOutput;
                        question.clear();
                        continue;
                    }
                    _ if wait_count.load(Ordering::Relaxed) >= options.timeout => {
                        if let Err(err) = sender.send(State::TimedOut) {
                            log::error!("Could send timeout error: {}", err);
                            return;
                        }
                    }
                    _ => {}
                },
                Err(TryRecvError::Disconnected) => {
                    if let Err(err) = sender.send(State::Completed) {
                        log::error!("Failed to send process end event {err}");
                    }
                    return;
                }
            }
        }
    });
}

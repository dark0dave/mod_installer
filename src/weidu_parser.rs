use std::{
    cmp::max,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc::{Receiver, Sender, TryRecvError},
    },
    thread,
};

use config::{parser_config::ParserConfig, state::State};

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
    timeout: usize,
) {
    let mut current_state = ParserState::LookingForInterestingOutput;
    let mut buffer = vec![];
    let mut question = vec![];
    sender
        .send(State::InProgress)
        .expect("Failed to send process start event");
    thread::spawn(move || {
        loop {
            match receiver.try_recv() {
                Ok(string) => match current_state {
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
                        let installer_state = parser_config.detect_weidu_finished_state(&string);
                        if installer_state != State::InProgress {
                            sender
                                .send(installer_state)
                                .expect("Failed to send process error event");
                            break;
                        }
                        if parser_config.string_looks_like_question(&string) {
                            log::debug!(
                                "Changing parser state to '{:?}' due to line {}",
                                ParserState::CollectingQuestion,
                                string
                            );
                            current_state = ParserState::CollectingQuestion;
                            question.push(string.clone());
                        }
                        if !string.trim().is_empty() {
                            log::info!("{string}");
                            buffer.push(string);
                        }
                    }
                },
                Err(TryRecvError::Empty) => match current_state {
                    ParserState::CollectingQuestion => {
                        log::debug!(
                            "Changing parser state to '{:?}'",
                            ParserState::WaitingForMoreQuestionContent
                        );
                        current_state = ParserState::WaitingForMoreQuestionContent;
                    }
                    ParserState::WaitingForMoreQuestionContent => {
                        log::debug!("No new weidu output, sending question to user");
                        let question_start = buffer
                            .iter()
                            .position(|n| n == question.first().unwrap_or(&"".to_string()))
                            .unwrap_or(0);
                        let out = buffer
                            .get(max(question_start - 5, 0_usize)..)
                            .unwrap_or(&question)
                            .iter()
                            .fold("".to_string(), |a, b| format!("{}\n{}", a, b));
                        sender
                            .send(State::RequiresInput { question: out })
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

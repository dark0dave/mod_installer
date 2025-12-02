use std::{
    collections::VecDeque,
    mem,
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
    sender
        .send(State::InProgress)
        .expect("Failed to send process start event");
    thread::spawn(move || {
        let mut question = String::new();
        let messages_to_store = 3;
        let mut last_few_lines = VecDeque::<String>::with_capacity(messages_to_store);
        loop {
            match receiver.try_recv() {
                Ok(string) => {
                    if last_few_lines.len() == messages_to_store {
                        last_few_lines.pop_front();
                    }
                    last_few_lines.push_back(string.clone());

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
                                question.push_str(&string);
                                current_state = ParserState::CollectingQuestion;
                            }
                        }
                        ParserState::LookingForInterestingOutput => {
                            let installer_state =
                                parser_config.detect_weidu_finished_state(&string);
                            if installer_state != State::InProgress {
                                sender
                                    .send(installer_state)
                                    .expect("Failed to send process error event");
                                break;
                            }
                            if parser_config
                                .string_looks_like_weidu_requested_input_explicidly(&string)
                            {
                                let prev_lines = last_few_lines.drain(..).fold(
                                    String::new(),
                                    |mut acc, chunk| {
                                        acc.push_str(&chunk);

                                        acc
                                    },
                                );
                                log::warn!(
                                    "Weidu unexpectedly requested input, sending previous lines to user",
                                );
                                send_question_to_user(&sender, &mut current_state, prev_lines);
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
                    }
                }
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
                        send_question_to_user(
                            &sender,
                            &mut current_state,
                            mem::take(&mut question),
                        );
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

fn send_question_to_user(
    sender: &Sender<State>,
    current_state: &mut ParserState,
    question: String,
) {
    sender
        .send(State::RequiresInput { question: question })
        .expect("Failed to send question");
    *current_state = ParserState::LookingForInterestingOutput;
}

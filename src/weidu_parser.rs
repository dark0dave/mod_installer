use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc::{Receiver, Sender, TryRecvError},
    },
    thread,
};

use config::{parser_config::ParserConfig, state::State};

const MAX_QUESTION_LINES: usize = 2000;
const MIN_IDLE_TICKS: usize = 3;

fn question_has_choices(lines: &VecDeque<String>) -> bool {
    lines.iter().any(|line| is_choice_line(line))
}

fn is_choice_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with('[') && lower.contains(']') {
        return true;
    }
    if trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
        && (trimmed.contains(')') || trimmed.contains(']'))
    {
        return true;
    }
    lower.starts_with("please choose one of the following")
        || lower.starts_with("please enter number")
        || lower.starts_with("please select")
}

fn should_emit_immediately(line: &str) -> bool {
    let lower = line.trim().to_ascii_lowercase();
    if lower.contains("is this correct?") {
        return true;
    }
    if lower.contains("answer [y]es or [n]o") {
        return true;
    }
    if lower.contains("[y]es") && lower.contains("[n]o") {
        return true;
    }
    if lower.contains("[a]ccept") || lower.contains("[r]etry") || lower.contains("[c]ancel") {
        return true;
    }
    if lower.contains("accept") && lower.contains("retry") && lower.contains("cancel") {
        return true;
    }
    if lower.starts_with("please select") {
        return true;
    }
    if lower.starts_with("please enter number") {
        return true;
    }
    if lower.starts_with("please enter") {
        return true;
    }
    if lower.contains("enter a new") {
        return true;
    }
    if lower.contains("leave blank") {
        return true;
    }
    false
}

fn build_question_block(lines: &VecDeque<String>) -> String {
    let mut out = String::new();
    let mut last: Option<&str> = None;
    for line in lines {
        let s = line.as_str();
        if last == Some(s) {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(s);
        last = Some(s);
    }
    out
}

pub(crate) fn parse_raw_output(
    sender: Sender<State>,
    receiver: Receiver<String>,
    parser_config: Arc<ParserConfig>,
    wait_count: Arc<AtomicUsize>,
    timeout: usize,
) {
    let mut buffer: VecDeque<String> = VecDeque::new();
    let mut pending_prompt = false;
    let mut idle_ticks: usize = 0;
    let mut last_wait_count = wait_count.load(Ordering::Relaxed);

    sender
        .send(State::InProgress)
        .expect("Failed to send process start event");

    thread::spawn(move || {
        loop {
            match receiver.try_recv() {
                Ok(string) => {
                    let installer_state = parser_config.detect_weidu_finished_state(&string);
                    if installer_state != State::InProgress {
                        sender
                            .send(installer_state)
                            .expect("Failed to send process error event");
                        break;
                    }

                    if !string.trim().is_empty() {
                        log::info!("{string}");
                        buffer.push_back(string.clone());
                        if buffer.len() > MAX_QUESTION_LINES {
                            buffer.pop_front();
                        }
                    }

                    let immediate = should_emit_immediately(&string);
                    if parser_config.string_looks_like_question(&string) || immediate {
                        pending_prompt = true;
                        idle_ticks = 0;
                        if immediate {
                            let out = build_question_block(&buffer);
                            sender
                                .send(State::RequiresInput { question: out })
                                .expect("Failed to send question");
                            pending_prompt = false;
                        }
                    }
                }
                Err(TryRecvError::Empty) => {
                    let current_wait = wait_count.load(Ordering::Relaxed);
                    if current_wait == last_wait_count {
                        continue;
                    }
                    last_wait_count = current_wait;
                    idle_ticks += 1;

                    if pending_prompt {
                        if question_has_choices(&buffer) || idle_ticks >= MIN_IDLE_TICKS {
                            let out = build_question_block(&buffer);
                            sender
                                .send(State::RequiresInput { question: out })
                                .expect("Failed to send question");
                            pending_prompt = false;
                            idle_ticks = 0;
                        }
                    }

                    if wait_count.load(Ordering::Relaxed) >= timeout {
                        sender
                            .send(State::TimedOut)
                            .expect("Could send timeout error");
                    }
                }
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

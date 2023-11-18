use core::time;
use std::{
    io::{self, BufRead, BufReader, ErrorKind, Write},
    panic,
    path::PathBuf,
    process::{Child, ChildStdout, Command, Stdio},
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
};

use crate::mod_component::ModComponent;

pub fn get_user_input() -> String {
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap_or_default();
    input.to_string()
}

fn generate_args(weidu_mod: &ModComponent, language: &str) -> Vec<String> {
    format!("{mod_name}/{mod_tp_file} --autolog --force-install {component} --use-lang {game_lang} --language {mod_lang}",
        mod_name = weidu_mod.name,
        mod_tp_file = weidu_mod.tp_file,
        component = weidu_mod.component,
        mod_lang = weidu_mod.lang,
        game_lang = language
    )
    .split(' ')
    .map(|x|x.to_string())
    .collect()
}

pub enum InstallationResult {
    Success,
    Warnings,
    Fail(String),
}

pub fn install(
    weidu_binary: &PathBuf,
    game_directory: &PathBuf,
    weidu_mod: &ModComponent,
    language: &str,
) -> InstallationResult {
    let weidu_args = generate_args(weidu_mod, language);
    let mut command = Command::new(weidu_binary);
    let weidu_process = command.current_dir(game_directory).args(weidu_args);

    let child = weidu_process
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn weidu process");

    handle_io(child)
}

#[derive(Debug)]
enum ProcessStateChange {
    RequiresInput { question: String },
    InProgress,
    Completed,
    CompletedWithErrors { error_details: String },
    CompletedWithWarnings,
}

pub fn handle_io(mut child: Child) -> InstallationResult {
    let mut weidu_stdin = child.stdin.take().unwrap();
    let output_lines_receiver = create_output_reader(child.stdout.take().unwrap());
    let parsed_output_receiver = create_parsed_output_receiver(output_lines_receiver);

    let mut wait_counter = 0;
    loop {
        match parsed_output_receiver.try_recv() {
            Ok(state) => {
                log::debug!("Current installer state is {:?}", state);
                match state {
                    ProcessStateChange::Completed => {
                        log::debug!("Weidu process completed");
                        break;
                    }
                    ProcessStateChange::CompletedWithErrors { error_details } => {
                        log::debug!("Weidu process seem to have completed with errors");
                        weidu_stdin
                            .write_all("\n".as_bytes())
                            .expect("Failed to send final ENTER to weidu process");
                        return InstallationResult::Fail(error_details);
                    }
                    ProcessStateChange::CompletedWithWarnings => {
                        log::debug!("Weidu process seem to have completed with warnings");
                        weidu_stdin
                            .write_all("\n".as_bytes())
                            .expect("Failed to send final ENTER to weidu process");
                        return InstallationResult::Warnings;
                    }
                    ProcessStateChange::InProgress => {
                        log::debug!("In progress...");
                    }
                    ProcessStateChange::RequiresInput { question } => {
                        log::info!("User Input required");
                        log::info!("Question is");
                        log::info!("{}\n", question);
                        log::info!("Please do so something!");
                        let user_input = get_user_input();
                        log::debug!("Read user input {}, sending it to process ", user_input);
                        weidu_stdin.write_all(user_input.as_bytes()).unwrap();
                        log::debug!("Input sent");
                    }
                }
            }
            Err(TryRecvError::Empty) => {
                print!("Waiting for child process to end");
                print!("{}\r", ".".repeat(wait_counter));
                std::io::stdout().flush().expect("Failed to flush stdout");

                wait_counter += 1;
                wait_counter %= 10;
                sleep(500);

                print!("\r                                                                   \r");
                std::io::stdout().flush().expect("Failed to flush stdout");
            }
            Err(TryRecvError::Disconnected) => break,
        }
    }
    InstallationResult::Success
}

#[derive(Debug)]
enum ParserState {
    CollectingQuestion,
    WaitingForMoreQuestionContent,
    LookingForInterestingOutput,
}

fn create_parsed_output_receiver(
    raw_output_receiver: Receiver<String>,
) -> Receiver<ProcessStateChange> {
    let (sender, receiver) = mpsc::channel::<ProcessStateChange>();
    parse_raw_output(sender, raw_output_receiver);
    receiver
}

fn parse_raw_output(sender: Sender<ProcessStateChange>, receiver: Receiver<String>) {
    let mut current_state = ParserState::LookingForInterestingOutput;
    let mut question = String::new();
    sender
        .send(ProcessStateChange::InProgress)
        .expect("Failed to send process start event");
    thread::spawn(move || loop {
        match receiver.try_recv() {
            Ok(string) => match current_state {
                ParserState::CollectingQuestion | ParserState::WaitingForMoreQuestionContent => {
                    if string_looks_like_weidu_is_doing_something_useful(&string) {
                        log::debug!(
                            "Weidu seems to know an answer for the last question, ignoring it"
                        );
                        current_state = ParserState::LookingForInterestingOutput;
                        question.clear();
                    } else {
                        log::debug!("Appending line '{}' to user question", string);
                        question.push_str(string.as_str());
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
                    } else {
                        log::debug!("Ignoring line {}", string);
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
                            .send(ProcessStateChange::RequiresInput { question })
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
                    .send(ProcessStateChange::Completed)
                    .expect("Failed to send provess end event");
                break;
            }
        }
    });
}

fn detect_weidu_finished_state(string: &str) -> Option<ProcessStateChange> {
    if string_looks_like_weidu_completed_with_errors(string) {
        Some(ProcessStateChange::CompletedWithErrors {
            error_details: string.trim().to_string(),
        })
    } else if string_looks_like_weidu_completed_with_warnings(string) {
        Some(ProcessStateChange::CompletedWithWarnings)
    } else {
        None
    }
}

fn string_looks_like_question(string: &str) -> bool {
    let lowercase_string = string.trim().to_lowercase();
    !lowercase_string.contains("installing")
        && (lowercase_string.contains("choice")
            || lowercase_string.starts_with("choose")
            || lowercase_string.starts_with("select")
            || lowercase_string.starts_with("do you want")
            || lowercase_string.starts_with("would you like")
            || lowercase_string.starts_with("enter"))
        || lowercase_string.ends_with('?')
        || lowercase_string.ends_with(':')
}

fn string_looks_like_weidu_is_doing_something_useful(string: &str) -> bool {
    let lowercase_string = string.trim().to_lowercase();
    lowercase_string.contains("copying")
        || lowercase_string.contains("copied")
        || lowercase_string.contains("installing")
        || lowercase_string.contains("installed")
        || lowercase_string.contains("patching")
        || lowercase_string.contains("patched")
        || lowercase_string.contains("processing")
        || lowercase_string.contains("processed")
}

fn string_looks_like_weidu_completed_with_errors(string: &str) -> bool {
    let lowercase_string = string.trim().to_lowercase();
    lowercase_string.contains("not installed due to errors")
}

fn string_looks_like_weidu_completed_with_warnings(string: &str) -> bool {
    let lowercase_string = string.trim().to_lowercase();
    lowercase_string.contains("installed with warnings")
}

fn create_output_reader(out: ChildStdout) -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    let mut buffered_reader = BufReader::new(out);
    thread::spawn(move || loop {
        let mut line = String::new();
        match buffered_reader.read_line(&mut line) {
            Ok(0) => {
                log::debug!("Process ended");
                break;
            }
            Ok(_) => {
                log::debug!("Got line from process: '{}'", line);
                tx.send(line).expect("Failed to sent process output line");
            }
            Err(ref e) if e.kind() == ErrorKind::InvalidData => {
                // sometimes there is a non-unicode gibberish in process output, it
                // does not seem to be an indicator of error or break anything, ignore it
                log::warn!("Failed to read weidu output");
            }
            Err(details) => {
                log::error!("Failed to read process output, error is '{:?}'", details);
                panic!("Failed to read process output, error is '{:?}'", details);
            }
        }
    });
    rx
}

fn sleep(millis: u64) {
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}

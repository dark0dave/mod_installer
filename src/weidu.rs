use core::time;
use std::{
    io::{self, BufRead, BufReader, Write},
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
    log::debug!("User input: {}", input);

    input.to_string()
}

fn generate_args(weidu_mod: &ModComponent, language: &str) -> Vec<String> {
    format!("{mod_name}/{mod_tp_file} --autolog --force-install {component} --use-lang {game_lang} --language {mod_lang}", mod_name = weidu_mod.name, mod_tp_file = weidu_mod.tp_file, component = weidu_mod.component, mod_lang = weidu_mod.lang, game_lang = language).split(' ').map(|x|x.to_string()).collect()
}

pub fn install(
    weidu_binary: &PathBuf,
    game_directory: &PathBuf,
    weidu_mod: &ModComponent,
    language: &str,
) -> Result<(), String> {
    let weidu_args = generate_args(weidu_mod, language);
    let mut command = Command::new(weidu_binary);
    let weidu_process = command.current_dir(game_directory).args(weidu_args.clone());

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
}

pub fn handle_io(mut child: Child) -> Result<(), String> {
    let mut writer = child.stdin.take().unwrap();

    let child_state_channel = create_output_reader(child.stdout.take().unwrap());
    let (parsed_output_sender, parsed_output_receiver) = mpsc::channel::<ProcessStateChange>();
    parse_output(parsed_output_sender, child_state_channel);
    let mut wait_counter = 0;
    loop {
        match parsed_output_receiver.try_recv() {
            Ok(   state) => {
              log::debug!("Current installer state is {:?}", state);
                match state {
                    ProcessStateChange::Completed => {
                        log::debug!("Weidu process completed");
                        break;
                    }
                    ProcessStateChange::CompletedWithErrors { error_details } => {
                        log::debug!("Weidu process seem to have completed with errors");
                        writer
                            .write("\n".as_bytes())
                            .expect("Failed to send final ENTER to weidu process");
                        return Err(error_details);
                    }
                    ProcessStateChange::InProgress => {
                        log::debug!("In progress...");
                    }
                    ProcessStateChange::RequiresInput { question } => {
                        println!("User Input required");
                        println!("Question is {}", question);
                        println!("Please do so something!");
                        let user_input = get_user_input();
                        println!("");
                        log::debug!("Read user input {}, sending it to process ", user_input);
                        writer.write_all(user_input.as_bytes()).unwrap();
                        log::debug!("Input sent");
                    }
                }
            }
            Err(TryRecvError::Empty) => {
                print!("No relevant output from child process, waiting");
                print!("{}", ".".repeat(wait_counter));
                wait_counter += 1;
                wait_counter %= 10;
                sleep(1000);
                print!("\r                                              X\r");
            }
            Err(TryRecvError::Disconnected) => break,
        }
    }

    match child.wait_with_output() {
        Ok(output) if !output.status.success() => {
            panic!("Something went wrong: {:#?}", output);
        }
        Err(err) => {
            panic!("Did not close properly: {}", err);
        }
        Ok(_) => {
            return Ok(());
        }
    }
}

#[derive(Debug)]
enum ParserState {
    CollectingQuestion,
    SleepingForQuestionToComplete,
    LookingForInterestingOutput,
}

fn parse_output(sender: Sender<ProcessStateChange>, receiver: Receiver<String>) {
    let mut current_state = ParserState::LookingForInterestingOutput;
    let mut question = String::new();
    sender
        .send(ProcessStateChange::InProgress)
        .expect("Failed to send process start event");
    thread::spawn(move || loop {
        match receiver.try_recv() {
            Ok(string) => match current_state {
                ParserState::CollectingQuestion | ParserState::SleepingForQuestionToComplete => {
                    if string_looks_like_processing(&string)
                    {
                        log::debug!("Weidu seems to know an answer for this question");
                        current_state = ParserState::LookingForInterestingOutput;
                        question.clear();
                    } else {
                        log::debug!("Appending line '{}' to user question", string);
                        question.push_str(string.as_str());
                        current_state = ParserState::CollectingQuestion;
                    }
                }
                ParserState::LookingForInterestingOutput => {
                    let lowercase_string = string.to_lowercase();
                    if lowercase_string.contains("not installed due to errors")
                    || lowercase_string.contains("installed with warnings") {
                        log::debug!("Weidu seems to have encountered errors during isntallation");
                        sender
                            .send(ProcessStateChange::CompletedWithErrors { error_details: string.trim().to_string() })
                            .expect("Failed to send process error event");
                        break;
                    } else if string_looks_like_question(&lowercase_string) {
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
                            ParserState::SleepingForQuestionToComplete
                        );
                        current_state = ParserState::SleepingForQuestionToComplete;
                    }
                    ParserState::SleepingForQuestionToComplete => {
                        log::debug!("No new weidu otput, sending question to user");
                        sender
                            .send(ProcessStateChange::RequiresInput {
                                question: question.clone(),
                            })
                            .expect("Failed to send question");
                        current_state = ParserState::LookingForInterestingOutput;
                        question.clear();
                    }
                    _ => {}
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

fn string_looks_like_question(string: &str) -> bool {
    let lowercase_string = string.to_lowercase();
    lowercase_string.contains("choice")
        || lowercase_string.starts_with("choose")
        || lowercase_string.starts_with("select")
        || lowercase_string.starts_with("do you want")
        || lowercase_string.starts_with("would you like")
        || lowercase_string.starts_with("enter")
}

fn string_looks_like_processing(string: &str) -> bool {
    let lowercase_string = string.to_lowercase();
    lowercase_string.contains("copying")
        || lowercase_string.contains("copied")
        || lowercase_string.contains("installing")
        || lowercase_string.contains("installed")
        || lowercase_string.contains("patching")
        || lowercase_string.contains("patched")
        || lowercase_string.contains("processing")
        || lowercase_string.contains("processed")
        || lowercase_string.ends_with(":\n")

}

fn create_output_reader(out: ChildStdout) -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    let mut buffered_reader = BufReader::new(out);
    thread::spawn(move || loop {
        let mut line = String::new();
        match buffered_reader.read_line(&mut line) {
            Ok(0) => {
                log::debug!("Weidu process ended");
                break;
            }
            Ok(_) => {
                log::debug!("Got line from weidu: '{}'", line);
                tx.send(line).expect("Failed to sent process output line");
            }
            Err(_) => {
                tx.send("Error".to_string()).expect("Oops");
            }
        }
    });
    rx
}

fn sleep(millis: u64) {
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}

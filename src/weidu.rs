use std::{
    io::{self, BufRead, BufReader, ErrorKind, Write},
    panic,
    path::PathBuf,
    process::{Child, ChildStdout, Command, Stdio},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, TryRecvError},
        Arc,
    },
    thread,
};

use crate::{
    mod_component::ModComponent, state::State, utils::sleep, weidu_parser::parse_raw_output,
};

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
    timeout: usize,
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

    handle_io(child, timeout)
}

pub fn handle_io(mut child: Child, timeout: usize) -> InstallationResult {
    let mut weidu_stdin = child.stdin.take().unwrap();
    let wait_counter = Arc::new(AtomicUsize::new(0));
    let raw_output_receiver = create_output_reader(child.stdout.take().unwrap());
    let (sender, parsed_output_receiver) = mpsc::channel::<State>();
    parse_raw_output(sender, raw_output_receiver, wait_counter.clone(), timeout);

    loop {
        match parsed_output_receiver.try_recv() {
            Ok(state) => {
                log::debug!("Current installer state is {:?}", state);
                match state {
                    State::Completed => {
                        log::debug!("Weidu process completed");
                        break;
                    }
                    State::CompletedWithErrors { error_details } => {
                        log::error!("Weidu process seem to have completed with errors");
                        weidu_stdin
                            .write_all("\n".as_bytes())
                            .expect("Failed to send final ENTER to weidu process");
                        return InstallationResult::Fail(error_details);
                    }
                    State::TimedOut => {
                        log::error!(
                            "Weidu process seem to have been running for {} seconds, exiting",
                            timeout
                        );
                        return InstallationResult::Fail("Timed out".to_string());
                    }
                    State::CompletedWithWarnings => {
                        log::warn!("Weidu process seem to have completed with warnings");
                        weidu_stdin
                            .write_all("\n".as_bytes())
                            .expect("Failed to send final ENTER to weidu process");
                        return InstallationResult::Warnings;
                    }
                    State::InProgress => {
                        log::debug!("In progress...");
                    }
                    State::RequiresInput { question } => {
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
                log::info!(
                    "{}\r",
                    ".".repeat(wait_counter.load(Ordering::Relaxed) % 10)
                );
                std::io::stdout().flush().expect("Failed to flush stdout");

                wait_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                sleep(1000);

                std::io::stdout().flush().expect("Failed to flush stdout");
            }
            Err(TryRecvError::Disconnected) => break,
        }
    }
    InstallationResult::Success
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
                log::debug!("{}", line);
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

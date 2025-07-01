use std::{
    io::{BufRead, BufReader, ErrorKind, Write},
    path::Path,
    process::{Child, ChildStdout, Command, Stdio},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, TryRecvError},
        Arc, RwLock,
    },
    thread,
};

use crate::{
    component::Component,
    config::{args::Options, parser_config::ParserConfig},
    state::State,
    utils::{get_user_input, sleep},
    weidu_parser::parse_raw_output,
};

fn generate_args(weidu_mod: &Component, weidu_log_mode: &str, language: &str) -> Vec<String> {
    format!("{mod_name}/{mod_tp_file} {weidu_log_mode} --force-install {component} --use-lang {game_lang} --language {mod_lang}",
        mod_name = weidu_mod.name,
        mod_tp_file = weidu_mod.tp_file,
        weidu_log_mode = weidu_log_mode,
        component = weidu_mod.component,
        mod_lang = weidu_mod.lang,
        game_lang = language
    )
    .split(' ')
    .map(|x|x.to_string())
    .collect()
}

pub(crate) enum InstallationResult {
    Success,
    Warnings,
    Fail(String),
}

pub(crate) fn install(
    game_directory: &Path,
    parser_config: Arc<ParserConfig>,
    weidu_mod: &Component,
    options: &Options,
) -> InstallationResult {
    let weidu_args = generate_args(weidu_mod, &options.weidu_log_mode, &options.language);
    let mut command = Command::new(options.weidu_binary.clone());
    let weidu_process = command.current_dir(game_directory).args(weidu_args);

    let child = weidu_process
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn weidu process");

    handle_io(child, parser_config, options.timeout, options.tick)
}

pub(crate) fn handle_io(
    mut child: Child,
    parser_config: Arc<ParserConfig>,
    timeout: usize,
    tick: u64,
) -> InstallationResult {
    let log = Arc::new(RwLock::new(String::new()));
    let mut weidu_stdin = child.stdin.take().unwrap();
    let wait_count = Arc::new(AtomicUsize::new(0));
    let raw_output_receiver = create_output_reader(child.stdout.take().unwrap());
    let (sender, parsed_output_receiver) = mpsc::channel::<State>();
    parse_raw_output(
        sender,
        raw_output_receiver,
        parser_config,
        wait_count.clone(),
        log.clone(),
        timeout,
        tick,
    );

    loop {
        match parsed_output_receiver.try_recv() {
            Ok(state) => {
                log::debug!("Current installer state is {state:?}");
                match state {
                    State::Completed => {
                        log::debug!("Weidu process completed");
                        break;
                    }
                    State::CompletedWithErrors { error_details } => {
                        log::error!("Weidu process seem to have completed with errors");
                        log::error!("Dumping log: {}", log.read().unwrap());
                        weidu_stdin
                            .write_all("\n".as_bytes())
                            .expect("Failed to send final ENTER to weidu process");
                        return InstallationResult::Fail(error_details);
                    }
                    State::TimedOut => {
                        log::error!(
                            "Weidu process seem to have been running for {timeout} seconds, exiting"
                        );
                        log::error!("Dumping log: {}", log.read().unwrap());
                        return InstallationResult::Fail("Timed out".to_string());
                    }
                    State::CompletedWithWarnings => {
                        log::warn!("Weidu process seem to have completed with warnings");
                        log::warn!("Dumping log: {}", log.read().unwrap());
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
                        log::info!("{question}\n");
                        let user_input = get_user_input();
                        log::debug!("Read user input {user_input}, sending it to process ");
                        weidu_stdin.write_all(user_input.as_bytes()).unwrap();
                        log::debug!("Input sent");
                    }
                }
            }
            Err(TryRecvError::Empty) => {
                log::info!("{}", ".".repeat(wait_count.load(Ordering::Relaxed) % 10));
                std::io::stdout().flush().expect("Failed to flush stdout");

                wait_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                sleep(tick);

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
                log::debug!("{line}");
                tx.send(line).expect("Failed to sent process output line");
            }
            Err(ref e) if e.kind() == ErrorKind::InvalidData => {
                // sometimes there is a non-unicode gibberish in process output, it
                // does not seem to be an indicator of error or break anything, ignore it
                log::warn!("Failed to read weidu output");
            }
            Err(details) => {
                log::error!("Failed to read process output, error is '{details:?}'");
                panic!("Failed to read process output, error is '{details:?}'");
            }
        }
    });
    rx
}

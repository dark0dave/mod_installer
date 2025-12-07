use std::{
    error::Error,
    io::{BufRead, BufReader, ErrorKind, Write},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc::{Receiver, TryRecvError, channel},
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

#[cfg(windows)]
pub(crate) const LINE_ENDING: &str = "\r\n";
#[cfg(not(windows))]
pub(crate) const LINE_ENDING: &str = "\n";

pub(crate) enum WeiduExitStatus {
    Success,
    Warnings(String),
}

pub(crate) type InstallationResult = Result<WeiduExitStatus, Box<dyn Error>>;

fn generate_args(weidu_mod: &Component, weidu_log_mode: &str, language: &str) -> Vec<String> {
    let mod_name = &weidu_mod.name;
    let mod_tp_file = &weidu_mod.tp_file;
    vec![
        format!("{mod_name}/{mod_tp_file}"),
        weidu_log_mode.to_string(),
        "--force-install".to_string(),
        weidu_mod.component.to_string(),
        "--use-lang".to_string(),
        language.to_string(),
        "--language".to_string(),
        weidu_mod.lang.to_string(),
        "--no-exit-pause".to_string(),
    ]
}

fn create_output_reader(out: ChildStdout) -> Receiver<String> {
    let (tx, rx) = channel::<String>();
    let mut buffered_reader = BufReader::new(out);
    thread::spawn(move || {
        loop {
            let mut lines = String::new();
            match buffered_reader.read_line(&mut lines) {
                Ok(0) => {
                    log::debug!("Process ended");
                    break;
                }
                Ok(_) => {
                    lines
                        .split(LINE_ENDING)
                        .filter(|line| !line.trim().is_empty())
                        .for_each(|line| {
                            log::trace!("Sending: `{line}`");
                            tx.send(line.to_string())
                                .expect("Failed to sent process output line");
                        });
                }
                Err(ref e) if e.kind() == ErrorKind::InvalidData => {
                    // sometimes there is a non-unicode gibberish in process output, it
                    // does not seem to be an indicator of error or break anything, ignore it
                    log::warn!("Failed to read weidu output");
                }
                Err(details) => {
                    log::error!("Failed to read process output, error is '{details:?}'");
                    panic!()
                }
            }
        }
    });
    rx
}

fn run_weidu(
    receiver_state: Result<State, TryRecvError>,
    mut weidu_stdin: ChildStdin,
    wait_count: Arc<AtomicUsize>,
    tick: u64,
) -> InstallationResult {
    loop {
        match receiver_state {
            Ok(ref state) => {
                log::debug!("Current installer state is {state:?}");
                match state {
                    State::Completed => {
                        log::debug!("Weidu process completed");
                        return Ok(WeiduExitStatus::Success);
                    }
                    State::CompletedWithWarnings { weidu_log } => {
                        log::warn!("Weidu process seem to have completed with warnings");
                        log::warn!("Dumping log: {weidu_log}");
                        return Ok(WeiduExitStatus::Warnings(
                            "Weidu process exited with warnings".to_string(),
                        ));
                    }
                    State::CompletedWithErrors { weidu_log } => {
                        log::error!("Weidu process seem to have completed with errors");
                        log::error!("Dumping log: {weidu_log}");
                        return Err(weidu_log.clone().into());
                    }
                    State::TimedOut { timeout, weidu_log } => {
                        log::error!(
                            "Weidu process seem to have been running for {timeout} seconds, exiting"
                        );
                        log::error!("Dumping log: {weidu_log}");
                        return Err("Timed out".into());
                    }
                    State::RequiresInput { question } => {
                        log::info!("User Input required");
                        log::info!("Question is");
                        log::info!("{question}\n");
                        let user_input = get_user_input();
                        log::debug!("Read user input {user_input}, sending it to process ");
                        weidu_stdin.write_all(user_input.as_bytes())?;
                    }
                    State::InProgress => {
                        log::debug!("In progress...");
                    }
                }
            }
            Err(TryRecvError::Empty) => {
                log::info!("{}", ".".repeat(wait_count.load(Ordering::Relaxed) % 10));
                std::io::stdout().flush().expect("Failed to flush stdout");

                wait_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                log::trace!("Receiver is sleeping");
                sleep(tick);

                std::io::stdout().flush().expect("Failed to flush stdout");
            }
            Err(TryRecvError::Disconnected) => {
                return Ok(WeiduExitStatus::Warnings(
                    "Weidu process exited with warnings".to_string(),
                ));
            }
        };
    }
}

pub(crate) fn handle_io(
    mut child: Child,
    parser_config: Arc<ParserConfig>,
    timeout: usize,
    tick: u64,
) -> InstallationResult {
    let weidu_stdin = child
        .stdin
        .take()
        .ok_or("Failed to get weidu standard in")?;
    let weidu_stdout = child
        .stdout
        .take()
        .ok_or("Failed to get weidu standard out")?;
    let raw_output_receiver = create_output_reader(weidu_stdout);
    let (sender, parsed_output_receiver) = channel::<State>();

    let wait_count = Arc::new(AtomicUsize::new(0));
    parse_raw_output(
        sender,
        raw_output_receiver,
        parser_config,
        wait_count.clone(),
        timeout,
    );

    let status = run_weidu(
        parsed_output_receiver.try_recv(),
        weidu_stdin,
        wait_count,
        tick,
    );
    match child.try_wait() {
        Ok(Some(exit)) => {
            log::debug!("Weidu exit status: {exit}");
        }
        Ok(None) => {
            log::debug!("Weidu exited, but could not get status.");
        }
        Err(err) => {
            log::error!("Failed to close weidu process: {err}");
        }
    };
    status
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

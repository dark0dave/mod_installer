use std::{
    error::Error,
    io::{BufRead, BufReader, ErrorKind, Write},
    path::Path,
    process::{Child, ChildStderr, ChildStdout, Command, Stdio},
    sync::{
        Arc, RwLock,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, TryRecvError},
    },
    thread,
};

use config::{args::Options, parser_config::ParserConfig, state::State};

use crate::{
    component::Component,
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

fn run(
    timeout: usize,
    tick: u64,
    mut weidu_stdin: std::process::ChildStdin,
    log: Arc<RwLock<String>>,
    parsed_output_receiver: Receiver<State>,
    wait_count: Arc<AtomicUsize>,
) -> Result<WeiduExitStatus, Box<dyn Error + 'static>> {
    let mut awaiting_input = false;
    loop {
        match parsed_output_receiver.try_recv() {
            Ok(state) => {
                log::debug!("Current installer state is {state:?}");
                match state {
                    State::Completed => {
                        log::debug!("Weidu process completed");
                        return Ok(WeiduExitStatus::Success);
                    }
                    State::CompletedWithWarnings => {
                        log::warn!("Weidu process seem to have completed with warnings");
                        if let Ok(weidu_log) = log.read() {
                            log::warn!("Dumping log: {weidu_log}");
                        }
                        return Ok(WeiduExitStatus::Warnings(
                            "Weidu process exited with warnings".to_string(),
                        ));
                    }
                    State::CompletedWithErrors { error_details } => {
                        log::error!("Weidu process seem to have completed with errors");
                        if let Ok(weidu_log) = log.read() {
                            log::error!("Dumping log: {weidu_log}");
                        }
                        return Err(error_details.into());
                    }
                    State::TimedOut => {
                        log::error!(
                            "Weidu process seem to have been running for {timeout} seconds, exiting"
                        );
                        if let Ok(weidu_log) = log.read() {
                            log::error!("Dumping log: {weidu_log}");
                        }
                        return Err("Timed out".into());
                    }
                    State::InProgress => {
                        log::debug!("In progress...");
                        awaiting_input = false;
                    }
                    State::RequiresInput { question } => {
                        log::info!("User Input required");
                        log::info!("Question is");
                        log::info!("{question}\n");
                        let user_input = get_user_input();
                        awaiting_input = false;
                        log::debug!("Read user input {user_input}, sending it to process ");
                        weidu_stdin.write_all(user_input.as_bytes())?;
                        log::debug!("Input sent");
                    }
                }
            }
            Err(TryRecvError::Empty) => {
                if !awaiting_input {
                    log::info!("{}", ".".repeat(wait_count.load(Ordering::Relaxed) % 10));
                    std::io::stdout().flush().expect("Failed to flush stdout");
                }

                wait_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                log::trace!("Receiver is sleeping");
                sleep(tick);

                if !awaiting_input {
                    std::io::stdout().flush().expect("Failed to flush stdout");
                }
            }
            Err(TryRecvError::Disconnected) => return Ok(WeiduExitStatus::Success),
        }
    }
}

fn create_output_reader(
    out: ChildStdout,
    err: Option<ChildStderr>,
    log: Arc<RwLock<String>>,
) -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();

    let tx_out = tx.clone();
    let log_out = log.clone();
    thread::spawn(move || read_stream("stdout", out, log_out, tx_out));

    if let Some(err) = err {
        let tx_err = tx.clone();
        let log_err = log.clone();
        thread::spawn(move || read_stream("stderr", err, log_err, tx_err));
    }

    rx
}

fn read_stream<R: std::io::Read>(
    label: &str,
    stream: R,
    log: Arc<RwLock<String>>,
    tx: mpsc::Sender<String>,
) {
    let mut buffered_reader = BufReader::new(stream);
    loop {
        let mut lines = String::new();
        match buffered_reader.read_line(&mut lines) {
            Ok(0) => {
                log::debug!("{label} ended");
                break;
            }
            Ok(_) => {
                if let Ok(mut writer) = log.write() {
                    writer.push_str(&lines);
                }
                lines
                    .split(LINE_ENDING)
                    .filter(|line| !line.trim().is_empty())
                    .for_each(|line| {
                        tx.send(line.to_string())
                            .expect("Failed to sent process output line");
                    });
            }
            Err(ref e) if e.kind() == ErrorKind::InvalidData => {
                log::warn!("Failed to read weidu output from {label}");
            }
            Err(details) => {
                panic!("Failed to read process output, error is '{details:?}'");
            }
        }
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
    let weidu_stderr = child.stderr.take();
    let log = Arc::new(RwLock::new(String::new()));
    let raw_output_receiver = create_output_reader(weidu_stdout, weidu_stderr, log.clone());
    let (sender, parsed_output_receiver) = mpsc::channel::<State>();

    let wait_count = Arc::new(AtomicUsize::new(0));
    parse_raw_output(
        sender,
        raw_output_receiver,
        parser_config,
        wait_count.clone(),
        timeout,
    );

    let result = run(
        timeout,
        tick,
        weidu_stdin,
        log,
        parsed_output_receiver,
        wait_count,
    );
    match child.try_wait() {
        Ok(Some(exit)) => {
            log::debug!("Weidu exit status: {exit}");
            if !exit.success() {
                if exit.code() == Some(3) {
                    return result;
                }
                return InstallationResult::Err(
                    format!("Weidu command failed with exit status: {exit}").into(),
                );
            }
            result
        }
        Ok(None) => {
            log::warn!("Weidu exited, but could not get status.");
            result
        }
        Err(err) => {
            log::error!("Failed to close weidu process: {err}");
            InstallationResult::Err(err.into())
        }
    }
}

fn generate_args(weidu_mod: &Component, weidu_log_mode: &str, language: &str) -> Vec<String> {
    let mod_name = &weidu_mod.name;
    let mod_tp_file = &weidu_mod.tp_file;
    let mut args = vec![
        format!("{mod_name}/{mod_tp_file}").to_lowercase(),
        "--force-install".to_string(),
        weidu_mod.component.to_string(),
        "--use-lang".to_string(),
        language.to_string(),
        "--language".to_string(),
        weidu_mod.lang.to_string(),
        "--no-exit-pause".to_string(),
    ];
    weidu_log_mode
        .split_ascii_whitespace()
        .for_each(|log_option| args.push(log_option.to_string()));
    args
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
    log::debug!(
        "cmd: {:?} {:?}",
        weidu_process.get_program(),
        weidu_process
            .get_args()
            .fold("".to_string(), |a, b| format!(
                "{} {:?}",
                a,
                b.to_os_string()
            ))
    );

    let child = weidu_process
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn weidu process");

    handle_io(child, parser_config, options.timeout, options.tick)
}

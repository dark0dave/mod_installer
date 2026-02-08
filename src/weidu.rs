use std::{
    error::Error,
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, RwLock,
        atomic::AtomicUsize,
        mpsc::{self, Receiver, TryRecvError},
    },
};

use config::{args::Options, log_options::LogOptions, parser_config::ParserConfig, state::State};

use crate::{
    component::Component,
    raw_reciever::create_raw_reciever,
    utils::{get_user_input, sleep},
    weidu_parser::parse_raw_output,
};

const EET_CHECK: &str = "Enter the full path to your BG:EE+SoD installation then press Enter.";

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
    bg1_game_directory: Option<&PathBuf>,
) -> Result<WeiduExitStatus, Box<dyn Error + 'static>> {
    let mut eet_check_completed = false;
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
                    }
                    State::RequiresInput { question }
                        if bg1_game_directory.is_some()
                            && !eet_check_completed
                            && question.contains(EET_CHECK) =>
                    {
                        log::info!("🚨🚨🚨DECTECTED EET INSTALL, AUTO FILL ENABLED🚨🚨🚨");
                        let pre_eet_directory = &format!(
                            "{}\n",
                            bg1_game_directory.as_ref().unwrap().to_string_lossy()
                        );
                        log::info!("Sending {}", pre_eet_directory);
                        weidu_stdin.write_all(pre_eet_directory.as_bytes())?;
                        eet_check_completed = true;
                        log::debug!("Input sent");
                    }
                    State::RequiresInput { question } => {
                        log::info!("User Input required");
                        log::info!("Question is");
                        log::info!("{question}\n");
                        let user_input = get_user_input(tick)?;
                        log::debug!("Read user input {user_input}, sending it to process ");
                        weidu_stdin.write_all(user_input.as_bytes())?;
                        log::debug!("Input sent");
                    }
                }
            }
            Err(TryRecvError::Empty) => {
                wait_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                log::trace!("Receiver is sleeping");
                sleep(tick);
            }
            Err(TryRecvError::Disconnected) => return Ok(WeiduExitStatus::Success),
        }
    }
}

pub(crate) fn handle_io(
    mut child: Child,
    parser_config: Arc<ParserConfig>,
    timeout: usize,
    tick: u64,
    lookback: usize,
    bg1_game_directory: Option<&PathBuf>,
) -> InstallationResult {
    let weidu_stdin = child
        .stdin
        .take()
        .ok_or("Failed to get weidu standard in")?;
    let weidu_stdout = child
        .stdout
        .take()
        .ok_or("Failed to get weidu standard out")?;
    let weidu_stderr = child
        .stderr
        .take()
        .ok_or("Failed to get weidu standard error")?;
    let log = Arc::new(RwLock::new(String::new()));
    let raw_output_receiver = create_raw_reciever(weidu_stdout, weidu_stderr, log.clone());
    let (sender, parsed_output_receiver) = mpsc::channel::<State>();

    let wait_count = Arc::new(AtomicUsize::new(0));
    parse_raw_output(
        tick,
        sender,
        raw_output_receiver,
        parser_config,
        wait_count.clone(),
        timeout,
        lookback,
    );

    let result = run(
        timeout,
        tick,
        weidu_stdin,
        log,
        parsed_output_receiver,
        wait_count,
        bg1_game_directory,
    );
    match child.try_wait() {
        Ok(Some(exit)) => {
            log::debug!("Weidu exit status: {exit}");
            if !exit.success() && exit.code() != Some(3) {
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

fn generate_args(
    weidu_mod: &Component,
    weidu_log_mode: Vec<LogOptions>,
    language: &str,
) -> Vec<String> {
    let mod_name = &weidu_mod.name;
    let mod_tp_file = &weidu_mod.tp_file;
    let component_name =
        format!("{mod_name}{}{mod_tp_file}", std::path::MAIN_SEPARATOR).to_lowercase();
    let mut args = vec![
        component_name.clone(),
        "--force-install".to_string(),
        weidu_mod.component.to_string(),
        "--use-lang".to_string(),
        language.to_string(),
        "--language".to_string(),
        weidu_mod.lang.to_string(),
        "--no-exit-pause".to_string(),
    ];
    let component_log = format!("{}-{}.log", mod_name, weidu_mod.component).to_lowercase();
    weidu_log_mode.into_iter().for_each(|log_option| {
        log_option
            .to_string(&component_log)
            .split(' ')
            .for_each(|option| args.push(option.to_string()))
    });
    args
}

pub(crate) fn install(
    game_directory: &Path,
    parser_config: Arc<ParserConfig>,
    weidu_mod: &Component,
    options: &Options,
    bg1_game_directory: Option<&PathBuf>,
) -> InstallationResult {
    let weidu_args = generate_args(weidu_mod, options.weidu_log_mode.clone(), &options.language);
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

    handle_io(
        child,
        parser_config,
        options.timeout,
        options.tick,
        options.lookback,
        bg1_game_directory,
    )
}

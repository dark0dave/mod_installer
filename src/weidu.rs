use std::{
    error::Error,
    io::{BufReader, Read, Write},
    path::Path,
    process::{Child, Command, Stdio},
    sync::{
        Arc, RwLock,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, TryRecvError},
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
    let mut weidu_stdin = child
        .stdin
        .take()
        .ok_or("Failed to get weidu standard in")?;
    let wait_count = Arc::new(AtomicUsize::new(0));
    let raw_output_receiver = create_output_reader(
        child
            .stdout
            .take()
            .ok_or("Failed to get weidu standard out")?,
        log.clone(),
    );
    let (sender, parsed_output_receiver) = mpsc::channel::<State>();
    parse_raw_output(
        sender,
        raw_output_receiver,
        parser_config,
        wait_count.clone(),
        timeout,
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
                        if let Ok(weidu_log) = log.read() {
                            log::error!("Dumping log: {weidu_log}");
                        }
                        match child.try_wait() {
                            Ok(exit) => {
                                log::debug!("Weidu exit status: {exit:?}");
                            }
                            Err(err) => {
                                log::error!("Failed to close weidu process: {err}");
                            }
                        };

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
                    State::CompletedWithWarnings => {
                        log::warn!("Weidu process seem to have completed with warnings");
                        if let Ok(weidu_log) = log.read() {
                            log::warn!("Dumping log: {weidu_log}");
                        }
                        return match child.try_wait() {
                            Ok(Some(exit)) => {
                                log::debug!("Weidu exit status: {exit}");
                                Ok(WeiduExitStatus::Warnings(
                                    "Weidu process exited with warnings".to_string(),
                                ))
                            }
                            Ok(None) => Ok(WeiduExitStatus::Warnings(
                                "Weidu process exited with warnings".to_string(),
                            )),
                            Err(err) => {
                                log::error!("Failed to close weidu process: {err}");
                                Err("Failed to close weidu process, exiting".into())
                            }
                        };
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
                        weidu_stdin.write_all(user_input.as_bytes())?;
                        log::debug!("Input sent");
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
            Err(TryRecvError::Disconnected) => break,
        }
    }
    Ok(WeiduExitStatus::Success)
}

fn create_output_reader<R: Read + Send + 'static>(
    out: R,
    log: Arc<RwLock<String>>,
) -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    let mut buffered_reader = BufReader::new(out);
    thread::spawn(move || {
        use std::io::Read;

        let mut byte_buffer_vec: Vec<u8> = Vec::new();
        let mut single_byte = [0u8; 1];

        loop {
            match buffered_reader.read(&mut single_byte) {
                Ok(0) => {
                    process_complete_line(&mut byte_buffer_vec, &log, &tx);
                    log::debug!("Process ended");
                    break;
                }
                Ok(_) => {
                    let byte = single_byte[0];
                    byte_buffer_vec.push(byte);
                    if byte == b'\n' || byte == 0x07 {
                        process_complete_line(&mut byte_buffer_vec, &log, &tx);
                    }
                }
                Err(e) => {
                    log::error!("Failed to read process output: {e:?}");
                    break;
                }
            }
        }
    });
    rx
}

fn send_forward(tx: &mpsc::Sender<String>, line: &String) {
    let trimmed = line.trim();
    if !trimmed.is_empty() {
        log::trace!("Sending: `{trimmed}`");
        tx.send(trimmed.to_string()).expect("Failed to send process output line");
    }
}

fn send_to_log(log: &Arc<RwLock<String>>, line: &str) {
    if !line.is_empty() {
        if let Ok(mut writer) = log.write() {
            writer.push_str(line);
        }
    }
}

fn process_complete_line(buffer: &mut Vec<u8>, log: &Arc<RwLock<String>>, tx: &mpsc::Sender<String>) {
    if buffer.is_empty() {
        return;
    }

    let bytes = std::mem::take(buffer);
    buffer.reserve(128);

    match String::from_utf8(bytes) {
        Ok(line) => {
            send_to_log(log, &line);
            send_forward(tx, &line);
        }
        Err(e) => {
            log::warn!("Failed to convert byte buffer to UTF-8 string: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::Duration;

    fn test_output_reader(input: &[u8]) -> Vec<String> {
        let (reader, mut writer) = os_pipe::pipe().unwrap();
        let log = Arc::new(RwLock::new(String::new()));
        
        let rx = create_output_reader(reader, log.clone());
        
        writer.write_all(input).unwrap();
        drop(writer);
        
        let mut results = Vec::new();
        loop {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(line) => results.push(line),
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => panic!("Timed out waiting for output"),
            }
        }
        results
    }

    #[test]
    fn test_output_reader_with_newline_delimiter() {
        let results = test_output_reader(b"Hello World\nSecond Line\n");
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], "Hello World");
        assert_eq!(results[1], "Second Line");
    }

    #[test]
    fn test_output_reader_with_bell_delimiter() {
        let results = test_output_reader(b"Prompt text\x07More text\x07");
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], "Prompt text\x07");
        assert_eq!(results[1], "More text\x07");
    }

    #[test]
    fn test_output_reader_with_mixed_delimiters() {
        let results = test_output_reader(b"Line with newline\nLine with bell\x07Another newline\n");
        
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], "Line with newline");
        assert_eq!(results[1], "Line with bell\x07");
        assert_eq!(results[2], "Another newline");
    }

    #[test]
    fn test_output_reader_with_utf8() {
        let results = test_output_reader("Hello 🎮 World\nCafé résumé\n".as_bytes());
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], "Hello 🎮 World");
        assert_eq!(results[1], "Café résumé");
    }

    #[test]
    fn test_output_reader_with_invalid_utf8() {
        let input = b"Valid line\nInvalid \xFF sequence\nValid again\n";
        let results = test_output_reader(input);
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], "Valid line");
        assert_eq!(results[1], "Valid again");
    }
}

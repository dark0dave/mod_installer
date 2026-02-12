use std::{
    io::{BufRead, BufReader, ErrorKind},
    process::{ChildStderr, ChildStdout},
    sync::{
        Arc, RwLock,
        mpsc::{self, Receiver, Sender},
    },
    thread,
};

fn read_stream<R: std::io::Read>(
    label: &str,
    stream: R,
    log: Arc<RwLock<String>>,
    sender: Sender<String>,
) {
    let mut buffered_reader = BufReader::new(stream);
    loop {
        let mut buf = vec![];
        match buffered_reader.read_until(b'\n', &mut buf) {
            Ok(0) => {
                log::debug!("{label} ended");
                break;
            }
            Ok(_) => {
                let line = std::str::from_utf8(&buf).unwrap_or_default();
                if let Ok(mut writer) = log.write() {
                    writer.push_str(line);
                }

                if let Err(err) = sender.send(line.to_string()) {
                    log::warn!("Failed to send line: {}, with error {}", line, err);
                }
            }
            Err(ref e) if e.kind() == ErrorKind::InvalidData => {
                log::warn!("Failed to read weidu {label}");
            }
            Err(details) => {
                panic!("Failed to read process output, error is '{details:?}'");
            }
        }
    }
}

pub(crate) fn create_raw_reciever(
    stdout: ChildStdout,
    stderr: ChildStderr,
    log: Arc<RwLock<String>>,
) -> Receiver<String> {
    let (sender, reciever) = mpsc::channel::<String>();
    let sender_stdout = sender.clone();
    let log_stdout = log.clone();
    thread::spawn(move || read_stream("stdout", stdout, log_stdout, sender_stdout));
    thread::spawn(move || read_stream("stderr", stderr, log, sender));

    reciever
}

use std::{
    io::{BufRead, BufReader, ErrorKind, Read},
    process::{ChildStderr, ChildStdout},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::internal_log::InternalLog;

fn read_stream<R: Read>(label: &str, stream: R, log: InternalLog, sender: Sender<String>) {
    let mut buffered_reader = BufReader::new(stream);
    loop {
        let mut buf = vec![];
        match buffered_reader.read_until(b'\n', &mut buf) {
            Ok(0) => {
                log::debug!("{label} ended");
                return;
            }
            Ok(_) => {
                if let Ok(line) = std::str::from_utf8(&buf) {
                    log.write(line);

                    if let Err(err) = sender.send(line.to_string()) {
                        log::warn!("Failed to send line: {}, with error {}", line, err);
                        return;
                    }
                } else {
                    log::warn!("Could not convert {:?} to string", buf);
                }
            }
            Err(ref e) if e.kind() == ErrorKind::InvalidData => {
                log::warn!("Failed to read weidu {label}");
            }
            Err(details) => {
                log::error!("Failed to read process output, error is '{details:?}'");
                return;
            }
        }
    }
}

pub(crate) fn create_raw_reciever(
    stdout: ChildStdout,
    stderr: ChildStderr,
    log: InternalLog,
) -> Receiver<String> {
    let (sender, receiver) = mpsc::channel::<String>();
    let sender_stdout = sender.clone();
    let log_stdout = log.clone();
    thread::spawn(move || read_stream("stdout", stdout, log_stdout, sender_stdout));
    thread::spawn(move || read_stream("stderr", stderr, log, sender));

    receiver
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::{error::Error, io::Write, time::Duration};

    fn test_output_reader(input: &[u8]) -> Result<Vec<String>, Box<dyn Error>> {
        let (sender, receiver) = mpsc::channel::<String>();
        let (reader, mut writer) = os_pipe::pipe()?;

        thread::spawn(move || read_stream("test", reader, InternalLog::new(), sender));

        writer.write(input)?;
        drop(writer);

        let mut results: Vec<_> = Vec::new();
        loop {
            match receiver.recv_timeout(Duration::from_secs(1)) {
                Ok(line) => results.push(line),
                Err(_) => break,
            }
        }

        Ok(results)
    }

    #[test]
    fn test_output_reader_with_newline_delimiter() -> Result<(), Box<dyn Error>> {
        let results = test_output_reader(b"Hello World\nSecond Line\n")?;
        let expected = vec!["Hello World\n".to_string(), "Second Line\n".to_string()];

        assert_eq!(results, expected);
        Ok(())
    }

    #[test]
    fn test_output_reader_with_utf8() -> Result<(), Box<dyn Error>> {
        let results = test_output_reader("Hello 🎮 World\nCafé résumé\n".as_bytes())?;
        let expected = vec!["Hello 🎮 World\n".to_string(), "Café résumé\n".to_string()];

        assert_eq!(results, expected);
        Ok(())
    }

    #[test]
    fn test_output_reader_with_invalid_utf8() -> Result<(), Box<dyn Error>> {
        let results = test_output_reader(b"Valid line\nInvalid \xFF sequence\nValid again\n")?;
        let expected = vec!["Valid line\n".to_string(), "Valid again\n".to_string()];

        assert_eq!(results, expected);
        Ok(())
    }
}

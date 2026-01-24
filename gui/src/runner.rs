use std::error::Error;
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[cfg(target_os = "windows")]
pub(crate) fn runner(cmd_string: &str) -> Result<(), Box<dyn Error>> {
    let mut process = Command::new("powershell")
        .args(&["-Command", "-"])
        .stderr(Stdio::piped())
        .spawn()?;
    let stdin = process.stdin.as_mut().ok_or("Could not get standard in")?;
    stdin.write_all(cmd_string.as_bytes())?;
    process.wait();
    Ok(())
}
#[cfg(not(target_os = "windows"))]
pub(crate) fn runner(cmd_string: &str) -> Result<(), Box<dyn Error>> {
    let process = Command::new(env!("TERM_PROGRAM").to_lowercase())
        .stderr(Stdio::piped())
        .output()?;
    Ok(())
}

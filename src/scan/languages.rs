use std::collections::HashSet;
use std::ffi::OsString;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::{error::Error, process::ChildStdout, process::Command, process::Stdio};

use crate::config::args::ScanLangauges;
use crate::utils::find_all_mods;

fn generate_args_for_list_lang(mod_path: &Path) -> Vec<OsString> {
  vec![
    "--nogame".into(),
    "--list-languages".into(),
    mod_path.into(),
    "--no-exit-pause".into(),
  ]
}

pub(crate) fn scan_for_langauges(
  mod_path: &Path,
  weidu_binary: &PathBuf,
  filter_by_selected_language: &str,
) -> Result<HashSet<String>, Box<dyn Error>> {
  let weidu_args_langs = generate_args_for_list_lang(mod_path);
  let mut run = Command::new(weidu_binary);
  let mod_root = mod_path
    .parent()
    .ok_or("tp2 file has no parent")?
    .parent()
    .ok_or("mod folder has no parent")?;
  log::debug!("{:?}", mod_root);
  let mut output = run
    .args(weidu_args_langs)
    .current_dir(mod_root)
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;
  if let Err(err) = output.try_wait() {
    log::warn!("Waiting for weidu process failed");
    return Err(Box::new(err));
  }
  let result: ChildStdout = output.stdout.ok_or("Failed to get output")?;
  let mut buffered_reader = BufReader::new(result);
  let mut buff = vec![];
  buffered_reader.read_to_end(&mut buff)?;
  let weidu_output = String::from_utf8(buff)?;
  log::trace!("{}", weidu_output);

  if weidu_output.is_empty() {
    log::warn!("Empty weidu response, {:?}", output.stderr);
    return Ok(HashSet::new());
  }
  Ok(
    weidu_output
      .split("\n")
      .flat_map(|lang| {
        if (*lang).starts_with(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'])
          && (*lang)
            .to_lowercase()
            .contains(&filter_by_selected_language.to_lowercase())
        {
          log::debug!("{}", lang);
          if let Some((lang_num, _)) = lang.split_once(":") {
            return Some(lang_num.to_string());
          }
        }
        None
      })
      .collect(),
  )
}

pub(crate) fn scan_langauges(command: &ScanLangauges) -> Result<(), Box<dyn Error>> {
  let mods = find_all_mods(&command.options.mod_directories, command.options.depth);
  log::trace!("{:?}", mods);

  for (_, weidu_mod) in mods {
    let langs = scan_for_langauges(
      &weidu_mod,
      &command.options.weidu_binary,
      &command.filter_by_selected_language,
    );
    println!("{:?} {:?}", weidu_mod, langs)
  }
  Ok(())
}

use std::{
  env::{split_paths, var_os},
  error::Error,
  path::PathBuf,
};

use clap::Command;
use url::Url;

use crate::config::{
  WEIDU_DL, WEIDU_FILE_NAME, WEIDU_FOLDER_PATH,
  args::{parse_absolute_path, path_exists_full},
  log_options::LogOptions,
};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Options {
  /// Absolute Path to weidu binary
  pub weidu_binary: PathBuf,

  /// Fetch weidu binary v249
  pub fetch_weidu_binary: bool,

  /// Path to mod directories
  pub mod_directories: Vec<PathBuf>,

  /// Depth to walk folder structure
  pub depth: usize,

  /// Weidu log setting "autolog,logapp,log-extern" is default
  pub weidu_log_mode: Vec<LogOptions>,

  /// Ocaml run parameters
  //  OCaml GC tuning — prevents 0xc0000005 segfaults on large installs:
  //   s=16M  = 128MB minor heap (64x default ~2MB, reduces minor GC frequency)
  //   o=500  = 500% space overhead (5x default, reduces major GC frequency)
  //   O=1000000 = disable heap compaction (compaction relocates memory blocks,
  //   can trigger stale-pointer crashes in WeiDU's unsafe-string code)
  pub ocamlrunparam: String,
}

impl clap::Args for Options {
  fn augment_args(cmd: Command) -> Command {
    cmd
      .arg(
        clap::Arg::new("weidu_binary")
          .env("WEIDU_BINARY")
          .short('w')
          .long("weidu-binary")
          .value_parser(parse_absolute_path)
          .required(false)
          .conflicts_with("fetch_weidu_binary"),
      )
      .arg(
        clap::Arg::new("fetch_weidu_binary")
          .env("FETCH_WEIDU_BINARY")
          .long("fetch-weidu-binary")
          .action(clap::ArgAction::SetTrue)
          .default_value("false")
          .required(false)
          .conflicts_with("weidu_binary"),
      )
      .arg(
        clap::Arg::new("mod_directories")
          .env("MOD_DIRECTORIES")
          .short('m')
          .long("mod-directories")
          .value_delimiter(',')
          .num_args(1..)
          .value_parser(path_exists_full)
          .required(false),
      )
      .arg(
        clap::Arg::new("depth")
          .env("DEPTH")
          .long("depth")
          .short('d')
          .required(false),
      )
      .arg(
        clap::Arg::new("weidu_log_mode")
          .env("WEIDU_LOG_MODE")
          .long("weidu-log-mode")
          .short('u')
          .value_delimiter(',')
          .num_args(1..)
          .value_parser(LogOptions::value_parser)
          .required(false),
      )
      .arg(
        clap::Arg::new("ocamlrunparam")
          .env("OCAMLRUNPARAM")
          .long("ocamlrunparam")
          .required(false),
      )
  }

  fn augment_args_for_update(cmd: Command) -> Command {
    cmd
  }
}

impl clap::FromArgMatches for Options {
  fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
    let fetch_weidu_binary = matches.get_flag("fetch_weidu_binary");

    let weidu_binary = if fetch_weidu_binary {
      download_weidu_bin()
    } else if let Ok(Some(bin)) = matches.try_get_one::<PathBuf>("weidu_binary") {
      Ok(bin.to_path_buf())
    } else {
      Ok(find_weidu_bin())
    }
    .map_err(|err| clap::Error::raw(clap::error::ErrorKind::Io, err.to_string()))?;

    let mod_directories = matches
      .get_many::<PathBuf>("mod_directories")
      .map(|vals| vals.cloned().collect::<Vec<_>>())
      .unwrap_or_else(current_work_dir);

    let depth = *matches.get_one::<usize>("depth").unwrap_or(&5);

    let weidu_log_mode = matches
      .get_many::<LogOptions>("weidu_log_mode")
      .map(|vals| vals.cloned().collect::<Vec<_>>())
      .unwrap_or_else(|| {
        vec![
          LogOptions::AutoLog,
          LogOptions::LogAppend,
          LogOptions::LogExternal,
        ]
      });

    let ocamlrunparam = matches
      .get_one::<String>("ocamlrunparam")
      .cloned()
      .unwrap_or_else(|| "s=16M,o=500,O=1000000".to_string());

    Ok(Self {
      weidu_binary,
      fetch_weidu_binary,
      mod_directories,
      depth,
      weidu_log_mode,
      ocamlrunparam,
    })
  }

  fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
    let mut matches = matches.clone();
    self.update_from_arg_matches_mut(&mut matches)
  }
}

pub(crate) fn find_weidu_bin() -> PathBuf {
  if let Some(paths) = var_os("PATH") {
    for path in split_paths(&paths) {
      let full_path = path.join(WEIDU_FILE_NAME);
      if full_path.is_file() && !full_path.is_dir() && full_path.exists() {
        return full_path;
      }
    }
  }
  PathBuf::new()
}

fn current_work_dir() -> Vec<PathBuf> {
  vec![std::env::current_dir().unwrap_or_default()]
}

fn download_weidu_bin() -> Result<PathBuf, Box<dyn Error>> {
  let url = Url::parse(WEIDU_DL)?;
  let mut zip_path = tempfile::tempfile()?;
  log::debug!("Downloading: {url}");
  reqwest::blocking::get(url.as_str())?.copy_to(&mut zip_path)?;
  let mut zip = zip::ZipArchive::new(zip_path)?;
  let dest = tempfile::tempdir()?.path().to_path_buf();
  zip.extract(dest.clone())?;
  let path_weidu_bin = dest.join(WEIDU_FOLDER_PATH).join(WEIDU_FILE_NAME);
  if !path_weidu_bin.exists() {
    return Err("Couldn't download weidu".into());
  }
  log::trace!("{:?}", path_weidu_bin);
  Ok(path_weidu_bin)
}

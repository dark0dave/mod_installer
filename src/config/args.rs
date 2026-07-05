use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;

use clap::Subcommand;
use clap::builder::ArgPredicate;
use clap::{Parser, builder::BoolishValueParser};

use crate::config::options::Options;
use crate::config::{CARGO_PKG_NAME, LONG};

use super::colors::styles;

// https://docs.rs/clap/latest/clap/_derive/index.html#arg-attributes
#[derive(Parser, Debug, PartialEq)]
#[command(
    name = CARGO_PKG_NAME,
    version,
    propagate_version = true,
    styles = styles(),
    about = format!("{}\n{}", LONG, env!("CARGO_PKG_DESCRIPTION"))
)]
pub struct Args {
  /// Install Type
  #[command(subcommand)]
  pub command: CommandType,
}

/// Type of Command
#[derive(Subcommand, Debug, PartialEq, Clone)]
pub enum CommandType {
  #[command()]
  Normal(Normal),
  #[command()]
  Eet(Eet),
  #[command()]
  Languages(ScanLangauges),
  #[command()]
  Components(ScanComponents),
}

/// Normal install for (BG1EE,BG2EE,IWDEE, EET)
#[derive(Parser, Debug, PartialEq, Clone)]
#[clap(short_flag = 'n')]
pub struct Normal {
  /// Path to target log
  #[clap(env, long, short = 'f', value_parser = path_must_exist, required = true)]
  pub log_file: PathBuf,

  /// Absolute Path to game directory
  #[clap(env, short, long, value_parser = parse_absolute_path, required = true)]
  pub game_directory: PathBuf,

  /// Instead of operating on an existing directory, create a new one with this flag as its name and then copy the original contents into it.
  #[clap(env, long, short = 'n', required = false)]
  pub generate_directory: Option<PathBuf>,

  /// Common Options
  #[clap(flatten)]
  pub options: Options,

  /// Install Options
  #[clap(flatten)]
  pub install_options: InstallOptions,
}

/// EET install for (eet) (BETA)
#[derive(Parser, Debug, PartialEq, Clone)]
#[clap(short_flag = 'e')]
pub struct Eet {
  /// Absolute Path to bg1ee game directory
  #[clap(env, short='1', long, value_parser = parse_absolute_path, required = true)]
  pub bg1_game_directory: PathBuf,

  /// Path to bg1ee weidu.log file
  #[clap(env, short='y', long, value_parser = path_must_exist, required = true)]
  pub bg1_log_file: PathBuf,

  /// Absolute Path to bg2ee game directory
  #[clap(env, short='2', long, value_parser = parse_absolute_path, required = true)]
  pub bg2_game_directory: PathBuf,

  /// Path to bg2ee weidu.log file
  #[clap(env, short='z', long, value_parser = path_must_exist, required = true)]
  pub bg2_log_file: PathBuf,

  /// Generates a new pre-eet directory.
  #[clap(env, short = 'p', long, value_parser = path_must_exist)]
  pub new_pre_eet_dir: Option<PathBuf>,

  /// Generates a new eet directory.
  #[clap(env, short = 'n', long, value_parser = path_must_exist)]
  pub new_eet_dir: Option<PathBuf>,

  /// Common Options
  #[clap(flatten)]
  pub options: Options,

  /// Install Options
  #[clap(flatten)]
  pub install_options: InstallOptions,
}

#[derive(Parser, Debug, PartialEq, Clone)]
#[clap(short_flag = 'l')]
pub struct ScanLangauges {
  /// filter by selected language
  #[clap(short, long, required = false, default_value = "")]
  pub filter_by_selected_language: String,

  /// Common Options
  #[clap(flatten)]
  pub options: Options,
}

#[derive(Parser, Debug, PartialEq, Clone)]
#[clap(short_flag = 'c')]
pub struct ScanComponents {
  /// filter by selected language
  #[clap(short, long, required = false, default_value = "")]
  pub filter_by_selected_language: String,

  /// Common Options
  #[clap(flatten)]
  pub options: Options,
}

#[derive(Parser, Debug, PartialEq, Clone, Default)]
pub struct InstallOptions {
  /// Game Language
  #[clap(short, long, default_value = "en_US")]
  pub language: String,

  /// Compare against installed weidu log, note this is best effort
  #[clap(
        env,
        short,
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetFalse,
        default_value_t = true,
        value_parser = BoolishValueParser::new(),
    )]
  pub skip_installed: bool,

  /// If a warning occurs in the weidu child process exit
  #[clap(
        env,
        short,
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetTrue,
        default_value_t = false,
        value_parser = BoolishValueParser::new(),
        conflicts_with = "never_abort",
    )]
  pub abort_on_warnings: bool,

  /// If an error occurs in the weidu child process continue
  #[clap(
        env,
        short = 'v',
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetTrue,
        default_value_t = false,
        value_parser = BoolishValueParser::new(),
        conflicts_with = "abort_on_warnings",
    )]
  pub never_abort: bool,

  /// Timeout time per mod in seconds, default is 3 hours or 9 if batch mode
  #[clap(
    env,
    short,
    long,
    default_value = "10800",
    default_value_if("batch_mode", ArgPredicate::IsPresent, "32400")
  )]
  pub timeout: usize,

  /// Strict Version and Component/SubComponent matching
  #[clap(
        env,
        short = 'x',
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetTrue,
        default_value_t = false,
        value_parser = BoolishValueParser::new(),
    )]
  pub strict_matching: bool,

  /// When a missing log is discovered ask the user for the download uri, download the mod and install it
  #[clap(
        env,
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetFalse,
        default_value_t = true,
        value_parser = BoolishValueParser::new(),
    )]
  pub download: bool,

  /// Force copy mod folder, even if the mod folder was found in the game directory
  #[clap(
        env,
        short,
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetTrue,
        default_value_t = false,
        value_parser = BoolishValueParser::new(),
    )]
  pub overwrite: bool,

  /// Strict weidu log checking
  #[clap(
        env,
        short,
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetFalse,
        default_value_t = true,
        conflicts_with = "batch_mode",
        value_parser = BoolishValueParser::new(),
    )]
  pub check_last_installed: bool,

  /// Tick
  #[clap(env, short = 'i', long, default_value_t = 500)]
  pub tick: u64,

  /// Lookback
  #[clap(env, short = '0', long, default_value_t = 10)]
  pub lookback: usize,

  /// Casefold only available for linux ext4
  #[clap(
        env,
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetTrue,
        default_value_t = false,
        value_parser = BoolishValueParser::new(),
    )]
  pub casefold: bool,

  /// Generic weidu args
  #[clap(short = 'k', long, use_value_delimiter = true, value_delimiter = ',')]
  pub generic_weidu_args: Vec<String>,

  /// Batch options
  #[clap(flatten)]
  pub batch: BatchOptions,
}

#[derive(Parser, Debug, PartialEq, Clone, Default)]
pub struct BatchOptions {
  /// Batch mode
  #[clap(
        env,
        long,
        num_args=0..=1,
        action = clap::ArgAction::SetTrue,
        default_value_t = false,
        conflicts_with = "check_last_installed",
        value_parser = BoolishValueParser::new(),
   )]
  pub batch_mode: bool,

  /// Batch size
  #[clap(env, long, default_value_t = 5, required = false)]
  pub batch_size: usize,

  /// Batch skip
  #[clap(
    env,
    long,
    use_value_delimiter = true,
    value_delimiter = ',',
    default_value = "setup-stratagems.tp2",
    ignore_case = true,
    required = false
  )]
  pub batch_skip: Vec<String>,
}

pub fn path_must_exist(arg: &str) -> Result<PathBuf, std::io::Error> {
  let path = PathBuf::from(arg);
  path.try_exists()?;
  Ok(path)
}

pub(crate) fn path_exists_full(arg: &str) -> Result<PathBuf, std::io::Error> {
  fs::canonicalize(path_must_exist(arg)?)
}

pub(crate) fn parse_absolute_path(arg: &str) -> Result<PathBuf, String> {
  let path = path_must_exist(arg).map_err(|err| err.to_string())?;
  fs::canonicalize(path).map_err(|err| err.to_string())
}

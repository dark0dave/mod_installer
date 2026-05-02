use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::{error::Error, path::Path, sync::Arc};

use crate::config::args::{Eet, Normal, Options};
use crate::config::parser_config::ParserConfig;
use crate::runner::{self, WeiduExitStatus};
use crate::utils::{copy_folder, mod_folder_present_in_game_directory};
use crate::utils::{delete_folder, get_last_installed, search_or_download};
use crate::weidu::batched_components::WeiduBatchedComponents;
use crate::weidu::install_block::WeiduInstallBlock;
use crate::weidu::install_order::WeiduBatchedInstallOrder;

pub(crate) fn normal_install(
  command: &Normal,
  parser_config: Arc<ParserConfig>,
  mod_folder_cache: &mut HashMap<OsString, PathBuf>,
) -> Result<(), Box<dyn Error>> {
  log::info!("Beginning normal install process");
  let game_directory = if let Some(new_directory) = &command.generate_directory {
    copy_folder(
      &command.game_directory,
      new_directory,
      command.options.casefold,
    )?;
    new_directory.clone()
  } else {
    command.game_directory.clone()
  };

  install(
    &command.log_file,
    &game_directory,
    &command.options,
    parser_config.clone(),
    None,
    mod_folder_cache,
  )
}

pub(crate) fn eet_install(
  command: &Eet,
  parser_config: Arc<ParserConfig>,
  mod_folder_cache: &mut HashMap<OsString, PathBuf>,
) -> Result<(), Box<dyn Error>> {
  log::info!("Beginning pre eet install process");
  let pre_eet_game_directory = if let Some(new_directory) = &command.new_pre_eet_dir {
    copy_folder(
      &command.bg1_game_directory,
      new_directory,
      command.options.casefold,
    )?;
    new_directory.clone()
  } else {
    command.bg1_game_directory.clone()
  };

  install(
    &command.bg1_log_file,
    &pre_eet_game_directory,
    &command.options,
    parser_config.clone(),
    None,
    mod_folder_cache,
  )?;

  log::info!("Beginning eet install process");
  let game_directory = if let Some(new_directory) = &command.new_eet_dir {
    copy_folder(
      &command.bg2_game_directory,
      new_directory,
      command.options.casefold,
    )?;
    new_directory.clone()
  } else {
    command.bg2_game_directory.clone()
  };
  install(
    &command.bg2_log_file,
    &game_directory,
    &command.options,
    parser_config.clone(),
    Some(&pre_eet_game_directory.to_path_buf()),
    mod_folder_cache,
  )
}

fn install(
  log_file_path: &Path,
  game_directory: &Path,
  options: &Options,
  parser_config: Arc<ParserConfig>,
  pre_eet_game_directory: Option<&PathBuf>,
  mod_folder_cache: &mut HashMap<OsString, PathBuf>,
) -> Result<(), Box<dyn Error>> {
  let mut components_to_be_installed: WeiduBatchedComponents =
    WeiduBatchedComponents::try_from(log_file_path.to_path_buf())?;
  if options.skip_installed {
    components_to_be_installed.remove_existing(options.strict_matching, game_directory)?;
  }
  let mods_to_be_installed: WeiduBatchedInstallOrder = if options.batch_mode {
    WeiduBatchedInstallOrder::batch(components_to_be_installed)?
  } else {
    WeiduBatchedInstallOrder::new(components_to_be_installed)
  };
  for components in mods_to_be_installed.into_iter() {
    let last_mod = if let Some(weidu_mod) = components.last() {
      weidu_mod
    } else {
      continue;
    };
    let mod_folder =
      if let Some(entry) = mod_folder_cache.get::<OsString>(&components.log_file_name().into()) {
        entry.to_path_buf()
      } else {
        let entry = match search_or_download(options, last_mod) {
          Ok(value) => value,
          Err(err) if options.never_abort => {
            log::error!("{:?}", err);
            log::info!("failed but never abort set, so continuing");
            continue;
          },
          Err(err) => return Err(err),
        };
        mod_folder_cache.insert(last_mod.tp_file.clone().into(), entry.clone());
        entry
      };

    log::debug!("Found mod folder {mod_folder:?}, for component {components:?}");

    if options.overwrite {
      delete_folder(game_directory.join(&last_mod.name))?;
    }

    if !mod_folder_present_in_game_directory(game_directory, &last_mod.name) {
      log::info!(
        "Copying mod directory, from {:?} to, {:?}",
        mod_folder,
        game_directory.join(&last_mod.name)
      );
      copy_folder(mod_folder, game_directory.join(&last_mod.name), false)?;
    }
    log::info!("Installing mod {:?}", &components);
    let bg1_game_directory = if last_mod
      .component_name
      .to_lowercase()
      .eq("eet core (resource importation)")
    {
      pre_eet_game_directory
    } else {
      None
    };
    let weidu_args = &components.generate_weidu_args(
      options.weidu_log_mode.clone(),
      &options.language,
      &options.generic_weidu_args,
    );
    match runner::spawn(
      game_directory,
      parser_config.clone(),
      options,
      weidu_args,
      bg1_game_directory,
    ) {
      Ok(WeiduExitStatus::Success) if options.check_last_installed && !options.never_abort => {
        if let Ok(last_installed) = get_last_installed(game_directory) {
          if last_installed.ne(last_mod) {
            return Err(format!(
                            "Last installed {last_installed:?} does not match component installed: {components:?}"
                        )
                        .into());
          }
          log::info!("Installed mod {:?}", &last_installed);
        } else {
          log::warn!("Could not open weidu log, to validate if last component was installed");
        }
      },
      Ok(WeiduExitStatus::Success) => {
        log::info!("Installed mod {:?}", components);
      },
      Ok(WeiduExitStatus::Warnings(msg)) if options.abort_on_warnings => {
        return Err(
          format!("Installed mod {components:?} with warnings: \n{msg}\n, stopping").into(),
        );
      },
      Ok(WeiduExitStatus::Warnings(msg)) => {
        log::warn!("Installed mod {components:?} with warnings:  \n{msg}\n")
      },
      Err(err) if options.never_abort => {
        log::error!("{:?}", err);
        log::info!("failed but never abort set, so continuing");
      },
      Err(err) => {
        return Err(err);
      },
    }
  }
  Ok(())
}

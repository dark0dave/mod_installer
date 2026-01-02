use crate::installer_type::InstallerType;

#[derive(Debug, Default, Clone)]
pub(crate) struct State {
    pub(crate) installer_bin: String,
    pub(crate) weidu_bin: String,
    pub(crate) mod_directory: String,
    pub(crate) installer_type: Option<InstallerType>,
    // Normal
    pub(crate) game_directory: String,
    pub(crate) weidu_log: String,
    // EET
    pub(crate) bg1_game_directory: String,
    pub(crate) bg1_weidu_log: String,
    pub(crate) bg2_game_directory: String,
    pub(crate) bg2_weidu_log: String,
    // All
    pub(crate) install_cmd: String,
}

impl State {
    pub(crate) fn new() -> Self {
        let mut out = Self::default();
        out.installer_type = Some(InstallerType::Normal);
        out.generate_installer_cmd();
        out
    }
    pub(crate) fn generate_installer_cmd(&mut self) {
        match self.installer_type {
            Some(InstallerType::EET) => {
                self.install_cmd = format!(
                    "{} -e -w {} -m {} -g {} -l {}",
                    self.installer_bin,
                    self.weidu_bin,
                    self.mod_directory,
                    self.game_directory,
                    self.weidu_log
                )
            }
            _ => {
                self.install_cmd = format!(
                    "{} -n -w {} -m {}",
                    self.installer_bin, self.weidu_bin, self.mod_directory,
                )
            }
        }
    }
}

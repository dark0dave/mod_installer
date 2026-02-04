use crate::state::shared::SharedState;

#[derive(Debug, Default, Clone)]
pub(crate) struct Normal {
    pub(crate) shared: SharedState,
    pub(crate) game_directory: String,
    pub(crate) weidu_log: String,
}

impl Normal {
    pub(crate) fn new(shared: &SharedState) -> Self {
        Self {
            shared: shared.clone(),
            ..Self::default()
        }
    }
    pub(crate) fn generate_installer_cmd(&mut self) {
        self.shared.install_cmd = format!(
            "{} -n -w {} -m {} -g {} -f {}",
            self.shared.installer_bin,
            self.shared.weidu_bin,
            self.shared.mod_directory,
            self.game_directory,
            self.weidu_log
        )
    }
}

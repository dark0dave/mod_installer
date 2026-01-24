use crate::state::shared::SharedState;

#[derive(Debug, Default, Clone)]
pub(crate) struct Eet {
    pub(crate) shared: SharedState,
    pub(crate) bg1_game_directory: String,
    pub(crate) bg1_weidu_log: String,
    pub(crate) bg2_game_directory: String,
    pub(crate) bg2_weidu_log: String,
}

impl Eet {
    pub(crate) fn new(shared: &SharedState) -> Self {
        Self {
            shared: shared.clone(),
            ..Self::default()
        }
    }
    pub(crate) fn generate_installer_cmd(&mut self) {
        self.shared.install_cmd = format!(
            "{} -e -w {} -m {} -1 {} -y {} -2 {} -z {}",
            self.shared.installer_bin,
            self.shared.weidu_bin,
            self.shared.mod_directory,
            self.bg1_game_directory,
            self.bg1_weidu_log,
            self.bg2_game_directory,
            self.bg2_weidu_log
        )
    }
}

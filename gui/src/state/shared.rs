use crate::installer_type::InstallerType;

#[derive(Debug, Default, Clone)]
pub(crate) struct SharedState {
    pub(crate) installer_bin: String,
    pub(crate) weidu_bin: String,
    pub(crate) mod_directory: String,
    pub(crate) installer_type: Option<InstallerType>,
    pub(crate) install_cmd: String,
}

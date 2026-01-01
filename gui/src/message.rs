use crate::installer_type::InstallerType;

#[derive(Debug, Clone)]
pub(crate) enum Message {
    InstallerPath(String),
    WeiduPath(String),
    ModDirectory(String),
    RadioSelected(InstallerType),
    GameDirectory(String),
    WeiduLogPath(String),
}

use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Metadata {
    pub(crate) mod_installer_version: String,
    pub(crate) created: SystemTime,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            mod_installer_version: env!("CARGO_PKG_VERSION").to_string(),
            created: SystemTime::now(),
        }
    }
}

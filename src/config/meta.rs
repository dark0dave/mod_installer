use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub mod_installer_version: String,
    pub created: SystemTime,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            mod_installer_version: env!("CARGO_PKG_VERSION").to_string(),
            created: SystemTime::now(),
        }
    }
}

use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub(crate) struct InternalLog(Arc<RwLock<String>>);

impl InternalLog {
    pub(crate) fn new() -> Self {
        Self(Arc::new(RwLock::new(String::new())))
    }
    pub(crate) fn write(&self, line: &str) {
        if !line.is_empty()
            && let Ok(mut writer) = self.0.write()
        {
            writer.push_str(line);
        }
    }
    pub(crate) fn read(&self) -> String {
        if let Ok(log) = self.0.read() {
            return log.clone();
        }
        String::new()
    }
}

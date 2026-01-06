use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FsWatcherSetting {
    pub directories: Vec<String>,
    pub files: Vec<String>,
}

impl Default for FsWatcherSetting {
    fn default() -> Self {
        Self {
            directories: vec![],
            files: vec![],
        }
    }
}

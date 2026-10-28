use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct FileMonitorIgnoreListWindows {
    #[serde(default)]
    pub system_dirs: Vec<String>,

    #[serde(default)]
    pub app_data_patterns: Vec<String>,

    #[serde(default)]
    pub system_patterns: Vec<String>,

    #[serde(default)]
    pub browser_patterns: Vec<String>,
}

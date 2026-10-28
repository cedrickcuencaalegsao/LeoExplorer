use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedFileExtension {
    pub extension: String,
    pub count: i32,
    pub file_type: String,
    pub is_selected: bool,
}

/// Payload from frontend for updating deep search settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSearchSettingsPayload {
    pub drives: Vec<String>,
    #[serde(default)]
    pub folders: Vec<String>,
    pub order: String,
    #[serde(rename = "fileTypes")]
    pub file_types: Vec<String>,
    #[serde(rename = "includeDevFiles")]
    pub include_dev_files: bool,
    #[serde(rename = "stopAfterMatch")]
    pub stop_after_match: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeepSearchSettings {
    pub thread_count_per_core: usize,
    pub stop_after_match: bool,
    pub include_dev_files: bool,
    pub supported_file_extensions: Vec<SupportedFileExtension>,
    pub drives: Vec<String>,
    #[serde(default)]
    pub folders: Vec<String>,
    pub order: String,
}

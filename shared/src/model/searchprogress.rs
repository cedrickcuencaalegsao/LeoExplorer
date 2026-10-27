use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchProgress {
    pub files_scanned: usize,
    pub files_matched: usize,
    pub current_file: String,
    pub total_files: Option<usize>,
    pub drives_completed: usize,
    pub total_drives: usize,
    pub duration_seconds: f32,
    pub files_per_second: f32,
}
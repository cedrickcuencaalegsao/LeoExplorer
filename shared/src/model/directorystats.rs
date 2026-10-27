use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default)]
pub struct DirectoryStats{
    pub file_count: u64,
    pub folder_count: u64,
    pub total_size_bytes: u64,
}

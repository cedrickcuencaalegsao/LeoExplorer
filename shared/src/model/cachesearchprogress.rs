use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSearchProgress {
    pub phase: String,
    pub message: String,
    pub roots_processed: usize,
    pub total_roots: usize,
    pub entries_visited: usize,
    pub matches: usize,
    pub duration_ms: u64,
}

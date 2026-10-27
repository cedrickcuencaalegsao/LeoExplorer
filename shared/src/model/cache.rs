use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Serialize, Deserialize)]
pub struct Cache {
    pub num_of_cores: usize,
    pub thread_count_per_core: usize,
}

static TREE_INSERT_SEQ: AtomicU64 = AtomicU64::new(0);
static TREE_INSERT_TOTAL: AtomicU64 = AtomicU64::new(0);

/// Reset counters before a full tree build; `total_expected` is the number of scanned paths
/// (each becomes one node in the adjacency map / tree).
pub fn reset_tree_build_progress(total_expected: u64) {
    TREE_INSERT_SEQ.store(0, Ordering::Relaxed);
    TREE_INSERT_TOTAL.store(total_expected, Ordering::Relaxed);
}

/// Log each `FileEntry` node materialized during cache tree construction (terminal progress).
pub fn log_cache_tree_insert(path_str: &str, is_dir: bool) {
    let i = TREE_INSERT_SEQ.fetch_add(1, Ordering::Relaxed) + 1;
    let total = TREE_INSERT_TOTAL.load(Ordering::Relaxed);
    let kind = if is_dir { "DIR " } else { "FILE" };
    if total > 0 {
        println!("[cache-tree] {i:>7}/{total:<7} {kind} {path_str}");
    } else {
        println!("[cache-tree] {i:>7} {kind} {path_str}");
    }
}

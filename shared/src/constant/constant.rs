use std::time::Duration;

pub const DIRECTORY_STATS_MAX_DEPTH: usize = 12;
pub const DIRECTORY_STATS_MAX_ENTRIES: usize = 250_000;
pub const DIRECTORY_STATS_MAX_DURATION: Duration = Duration::from_secs(3);
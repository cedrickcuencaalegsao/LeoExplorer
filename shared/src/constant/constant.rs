use std::time::Duration;
use std::sync::atomic::AtomicU8;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use crate::model::CacheData;

pub const DIRECTORY_STATS_MAX_DEPTH: usize = 12;
pub const DIRECTORY_STATS_MAX_ENTRIES: usize = 250_000;
pub const DIRECTORY_STATS_MAX_DURATION: Duration = Duration::from_secs(3);

pub const CACHE_FILE_NAME: &str = "leocache.bin.zst";
pub const CACHE_TEMP_FILE_NAME: &str = "leocache.bin.zst.tmp";
pub static CACHE_STATUS: AtomicU8 = AtomicU8::new(0);
pub static CACHE_MESSAGE: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
pub static GLOBAL_CACHE: Lazy<Mutex<Option<CacheData>>> = Lazy::new(|| Mutex::new(None));
use crate::model::CacheData;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::sync::atomic::AtomicU8;
use std::time::Duration;

pub const DIRECTORY_STATS_MAX_DEPTH: usize = 12;
pub const DIRECTORY_STATS_MAX_ENTRIES: usize = 250_000;
pub const DIRECTORY_STATS_MAX_DURATION: Duration = Duration::from_secs(3);

pub const LEO_EXPLORER: &str = ".leo_explorer";
pub const SETTINGS_FILE: &str = "settings.json";
pub const SETTINGS_FOLDER: &str = "settings";
pub const CACHE_FILE_NAME: &str = "leocache.bin.zst";
pub const CACHE_TEMP_FILE_NAME: &str = "leocache.bin.zst.tmp";
pub static CACHE_STATUS: AtomicU8 = AtomicU8::new(0);
pub static CACHE_MESSAGE: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
pub static GLOBAL_CACHE: Lazy<Mutex<Option<CacheData>>> = Lazy::new(|| Mutex::new(None));

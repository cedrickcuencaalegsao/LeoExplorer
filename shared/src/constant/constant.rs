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

pub const SCROLL_GUTTER: f32 = 12.0;
pub const CARD_PADDING_X: f32 = 14.0;
pub const CARD_PADDING_Y: f32 = 10.0;
pub const COLUMN_SPACING: f32 = 16.0;
pub const CARD_SPACING: f32 = 8.0;
pub const ICON_SIZE: f32 = 20.0;
pub const ICON_GAP: f32 = 10.0;
pub const LOCK_SIZE: f32 = 14.0;
pub const NAME_SIZE: f32 = 14.0;
pub const META_SIZE: f32 = 13.0;
pub const SAFETY: f32 = 6.0;

pub const NAME_PORTION: u16 = 4;
pub const DATE_PORTION: u16 = 3;
pub const PATH_PORTION: u16 = 5;

pub const TILE_WIDTH: f32 = 128.0;
pub const TILE_HEIGHT: f32 = 120.0;
pub const TILE_PADDING: f32 = 10.0;
pub const TILE_ICON: f32 = 48.0;


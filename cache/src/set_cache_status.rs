use std::sync::atomic::Ordering;
use shared::CACHE_STATUS;
use shared::CACHE_MESSAGE;

pub fn set_cache_status(flag: u8, msg: Option<String>) {
    CACHE_STATUS.store(flag, Ordering::SeqCst);
    if let Ok(mut lock) = CACHE_MESSAGE.lock() {
        *lock = msg;
    }
}
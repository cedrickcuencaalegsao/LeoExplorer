use shared::CacheData;
use shared::GLOBAL_CACHE;

pub fn update_global_cache(data: &CacheData) {
    if let Ok(mut guard) = GLOBAL_CACHE.lock() {
        *guard = Some(data.clone());
    }
}
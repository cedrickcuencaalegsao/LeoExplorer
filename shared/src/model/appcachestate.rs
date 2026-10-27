use super::InMemorCache;
use std::sync::Mutex;

pub struct AppCacheState {
    pub data: Mutex<InMemorCache>,
}

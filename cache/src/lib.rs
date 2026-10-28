mod build_tree_blazing_fast;
mod cache_build_scan_percent;
mod cache_parallelism;
mod create_cache;
mod read_thread_settings;
mod save_extensions_to_settings;
mod scan_directory_parallel;
mod scan_file_extensions_from_cache;
mod set_cache_status;
mod should_skip_path;
mod update_global_cache;
mod write_cache_to_disk;

use shared::CacheEvent;
use shared::Reporter;

use base_path::find_cache;

use std::thread;

pub fn init(report: Reporter) {
    thread::spawn(move || {
        if find_cache() {
            log::info!("Cache already exists. Using cached data.");
            set_cache_status::set_cache_status(0, Some("Cache Loaded".to_string()));
            report(CacheEvent::Ready(
                "Cache already exists. Using cached data.".to_string(),
            ));
        } else {
            log::info!("Cache not found. Creating cache file.");
            report(CacheEvent::Creating(
                "Cache not found. Creating cache file.".to_string(),
            ));
            set_cache_status::set_cache_status(1, Some("Creating cache file.".to_string()));
            create_cache::create_cache(report);
        }
    });
}

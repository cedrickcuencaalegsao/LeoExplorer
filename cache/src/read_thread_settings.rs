use shared::Settings;
use base_path::settings_path;
use std::fs;

pub fn read_thread_settings() -> (usize, usize) {
    let settings_path = settings_path();

    if let Ok(content) = fs::read_to_string(&settings_path) {
        if let Ok(settings) = serde_json::from_str::<Settings>(&content) {
            let num_cores = settings.cache.num_of_cores.max(1);
            let threads_per_core = settings.cache.thread_count_per_core.max(1);
            return (num_cores, threads_per_core);
        }
    }

    // Default: one worker per logical CPU (Rayon handles intra-scan parallelism).
    (num_cpus::get().max(1), 1)
}
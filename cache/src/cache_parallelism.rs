use super::read_thread_settings::read_thread_settings;

pub fn cache_parallelism() -> usize {
    let logical = num_cpus::get().max(1);
    let (configured_cores, tpc) = read_thread_settings();
    let configured = configured_cores.saturating_mul(tpc).max(1);
    logical.max(configured).min(256)
}

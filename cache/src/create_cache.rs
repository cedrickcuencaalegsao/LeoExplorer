use super::build_tree_blazing_fast::build_tree_blazing_fast;
use super::cache_build_scan_percent::cache_build_scan_percent;
use super::cache_parallelism::cache_parallelism;
use super::read_thread_settings::read_thread_settings;
use super::save_extensions_to_settings::save_extensions_to_settings;
use super::scan_directory_parallel::scan_directory_parallel;
use super::scan_file_extensions_from_cache::scan_file_extensions_from_cache;
use super::set_cache_status::set_cache_status;
use super::update_global_cache::update_global_cache;
use super::write_cache_to_disk::write_cache_to_disk;
use base_path::{base_path, cache_file_path};
use shared::CacheBuildProgress;
use shared::CacheData;
use shared::CacheEvent;
use shared::Reporter;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use threadpool::ThreadPool;

use shared::emit_cache_build_progress;

pub fn create_cache(report: Reporter) {
    report(CacheEvent::Started("Starting cache scan...".to_string()));
    set_cache_status(1, Some("Starting cache scan...".to_string()));

    let base_dir = base_path();

    if let Err(e) = fs::create_dir_all(&base_dir) {
        log::error!("Failed to create cache directory {:?}: {}", base_dir, e);
        report(CacheEvent::Error(format!(
            "Failed to create directory: {}",
            e
        )));
        set_cache_status(0, Some(format!("Failed to create directory: {}", e)));
        return;
    }

    #[cfg(target_os = "windows")]
    let drive_roots = get_windows_drives();

    #[cfg(target_os = "macos")]
    let drive_roots = vec![PathBuf::from("/Users"), PathBuf::from("/Applications")];

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let drive_roots = vec![PathBuf::from("/home")];

    let max_depth = 4;
    let drive_roots_bg = drive_roots.clone();
    let report_bg = report.clone();
    let total_roots: u32 = drive_roots_bg.iter().filter(|r| r.exists()).count() as u32;

    std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel::<Vec<PathBuf>>();

        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(num_cpus::get().max(1))
            .build_global();

        let pool_size = cache_parallelism().max(2);
        let (num_cores, threads_per_core) = read_thread_settings();
        let pool = ThreadPool::new(pool_size);

        log::info!(
            "Starting parallel cache scan: pool={} threads (settings {}×{}), Rayon={} logical CPUs; zstd write uses multi-threaded compression",
            pool_size,
            num_cores,
            threads_per_core,
            num_cpus::get()
        );
        let parallel_msg = format!(
            "Parallel scan: {} workers, {} CPU cores (Rayon + zstd MT)",
            pool_size,
            num_cpus::get()
        );
        report_bg(CacheEvent::Progress(parallel_msg.clone()));

        let start = std::time::Instant::now();
        let total_found = Arc::new(AtomicUsize::new(0));
        let roots_done = Arc::new(AtomicUsize::new(0));

        emit_cache_build_progress(
            &report_bg,
            CacheBuildProgress {
                phase: "scanning".to_string(),
                percent: 0,
                message: parallel_msg,
                drives_completed: Some(0),
                total_drives: Some(total_roots),
                paths_found: Some(0),
                duration_seconds: Some(0.0),
            },
        );

        for root in drive_roots_bg.into_iter() {
            if root.exists() {
                let txc = tx.clone();
                let r = root.clone();
                let report_worker = report_bg.clone();
                let counter = total_found.clone();
                let roots_done_c = roots_done.clone();
                let tr = total_roots;

                pool.execute(move || {
                    report_worker(CacheEvent::Progress(format!("Scanning: {}", r.display())));
                    log::info!("Scanning drive: {}", r.display());

                    let mut local_results = Vec::with_capacity(10000);
                    local_results.extend(scan_directory_parallel(&r, max_depth, 0));

                    let count = local_results.len();
                    counter.fetch_add(count, Ordering::Relaxed);

                    log::info!("Completed: {} - {} items", r.display(), count);
                    report_worker(CacheEvent::Progress(format!(
                        "Completed: {} ({} items)",
                        r.display(),
                        count
                    )));

                    let k = roots_done_c.fetch_add(1, Ordering::SeqCst) + 1;
                    let paths = counter.load(Ordering::Relaxed) as u64;
                    let pct = cache_build_scan_percent(k, tr);
                    emit_cache_build_progress(
                        &report_worker,
                        CacheBuildProgress {
                            phase: "scanning".to_string(),
                            percent: pct,
                            message: format!("Completed: {} ({} items)", r.display(), count),
                            drives_completed: Some(k as u32),
                            total_drives: Some(tr),
                            paths_found: Some(paths),
                            duration_seconds: Some(start.elapsed().as_secs_f64()),
                        },
                    );

                    let _ = txc.send(local_results);
                });
            }
        }

        drop(tx);
        pool.join();

        let estimated_total = total_found.load(Ordering::Relaxed);
        let mut scanned = Vec::with_capacity(estimated_total);

        for part in rx.iter() {
            scanned.extend(part);
        }

        let scan_duration = start.elapsed();
        let total = scanned.len();

        log::info!(
            "Scan completed in {:.2}s - {} paths found",
            scan_duration.as_secs_f64(),
            total
        );
        log::info!("Building tree structure...");

        let build_msg = format!("Building tree from {} items...", total);
        report_bg(CacheEvent::Progress(build_msg.clone()));
        emit_cache_build_progress(
            &report_bg,
            CacheBuildProgress {
                phase: "building_tree".to_string(),
                percent: 68,
                message: build_msg,
                drives_completed: Some(total_roots),
                total_drives: Some(total_roots.max(1)),
                paths_found: Some(total as u64),
                duration_seconds: Some(start.elapsed().as_secs_f64()),
            },
        );

        let tree_start = std::time::Instant::now();
        let roots = build_tree_blazing_fast(&scanned);
        let tree_duration = tree_start.elapsed();

        log::info!("Tree built in {:.2}s", tree_duration.as_secs_f64());

        let final_data = CacheData { roots };

        log::info!("Writing cache file (bincode + zstd)...");
        report_bg(CacheEvent::Progress("Writing cache file...".to_string()));
        emit_cache_build_progress(
            &report_bg,
            CacheBuildProgress {
                phase: "writing".to_string(),
                percent: 80,
                message: "Writing cache file...".to_string(),
                drives_completed: Some(total_roots),
                total_drives: Some(total_roots.max(1)),
                paths_found: Some(total as u64),
                duration_seconds: Some(start.elapsed().as_secs_f64()),
            },
        );

        let write_start = std::time::Instant::now();

        match write_cache_to_disk(&final_data) {
            Ok(_) => {
                let cache_file = cache_file_path();
                let write_duration = write_start.elapsed();
                let total_duration = start.elapsed();

                log::info!("Write completed in {:.2}s", write_duration.as_secs_f64());
                log::info!(
                    "TOTAL TIME: {:.2}s for {} paths",
                    total_duration.as_secs_f64(),
                    total
                );

                match fs::metadata(&cache_file) {
                    Ok(final_meta) => {
                        log::info!(
                            "Cache file: {:.2} MB",
                            final_meta.len() as f64 / 1_048_576.0
                        );

                        report_bg(CacheEvent::Completed(format!(
                            "Cache created in {:.1}s! {} paths",
                            total_duration.as_secs_f64(),
                            total
                        )));
                        set_cache_status(
                            0,
                            Some(format!(
                                "Cache created! {} paths in {:.1}s",
                                total,
                                total_duration.as_secs_f64()
                            )),
                        );

                        update_global_cache(&final_data);

                        let ext_msg = "Scanning file extensions...".to_string();
                        report_bg(CacheEvent::Progress(ext_msg.clone()));
                        emit_cache_build_progress(
                            &report_bg,
                            CacheBuildProgress {
                                phase: "extensions".to_string(),
                                percent: 96,
                                message: ext_msg,
                                drives_completed: Some(total_roots),
                                total_drives: Some(total_roots.max(1)),
                                paths_found: Some(total as u64),
                                duration_seconds: Some(start.elapsed().as_secs_f64()),
                            },
                        );
                        log::info!("Scanning file extensions from cache tree...");

                        let extensions = scan_file_extensions_from_cache(&final_data);
                        log::info!("Found {} unique file extensions", extensions.len());

                        save_extensions_to_settings(extensions);

                        report_bg(CacheEvent::ExtensionsReady(
                            "File extensions have been scanned and saved.".to_string(),
                        ));
                    }
                    Err(e) => {
                        log::warn!("Could not verify final cache file: {}", e);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to write cache: {}", e);
                report_bg(CacheEvent::Error(format!("Failed to write cache: {}", e)));
                set_cache_status(0, Some(format!("Failed to write cache: {}", e)));
            }
        }
    });
}

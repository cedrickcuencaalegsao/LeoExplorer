use shared::CacheBuildProgress;
use shared::CacheData;
use shared::SupportedFileExtension;
use shared::FileEntry;
use shared::Settings;
use shared::CacheEvent;
use shared::Reporter;
use shared::emit_cache_build_progress;

use base_path::{base_path, find_cache, settings_path, cache_file_path, cache_temp_file_path};

use num_cpus;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::path::{Path, PathBuf};
use threadpool::ThreadPool;

use shared::CACHE_STATUS;
use shared::CACHE_MESSAGE;
use shared::GLOBAL_CACHE;

pub fn initialize_cache_directory(report: Reporter) {
    thread::spawn(move || {
        if find_cache() {
            log::info!("Cache already exists. Using cached data.");
            set_cache_status(0, Some("Cache Loaded".to_string()));
            report(CacheEvent::Ready("Cache already exists. Using cached data.".to_string()));
        } else {
            log::info!("Cache not found. Creating cache file.");
            report(CacheEvent::Creating("Cache not found. Creating cache file.".to_string()));
            set_cache_status(1, Some("Creating cache file.".to_string()));
            create_cache(report);
        }
    });
}

fn set_cache_status(flag: u8, msg: Option<String>) {
    CACHE_STATUS.store(flag, Ordering::SeqCst);
    if let Ok(mut lock) = CACHE_MESSAGE.lock() {
        *lock = msg;
    }
}

fn create_cache(report: Reporter) {
    report(CacheEvent::Started("Starting cache scan...".to_string()));
    set_cache_status(1, Some("Starting cache scan...".to_string()));

    let base_dir = base_path();

    if let Err(e) = fs::create_dir_all(&base_dir) {
        log::error!("Failed to create cache directory {:?}: {}", base_dir, e);
        report(CacheEvent::Error(format!("Failed to create directory: {}", e)));
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
                        log::info!("Cache file: {:.2} MB", final_meta.len() as f64 / 1_048_576.0);

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

fn cache_parallelism() -> usize {
    let logical = num_cpus::get().max(1);
    let (configured_cores, tpc) = read_thread_settings();
    let configured = configured_cores.saturating_mul(tpc).max(1);
    logical.max(configured).min(256)
}

fn read_thread_settings() -> (usize, usize) {
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

fn scan_directory_parallel(dir: &Path, max_depth: u32, current_depth: u32) -> Vec<PathBuf> {
    if current_depth > max_depth {
        return Vec::new();
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut results = Vec::with_capacity(256);
    let mut subdirs = Vec::with_capacity(32);

    for entry in entries.flatten() {
        let path = entry.path();

        if should_skip_path(&path) {
            continue;
        }

        results.push(path.clone());

        if path.is_dir() {
            subdirs.push(path);
        }
    }

    if subdirs.len() > 1 && current_depth < max_depth {
        let sub_results: Vec<Vec<PathBuf>> = subdirs
            .par_iter()
            .map(|subdir| scan_directory_parallel(subdir, max_depth, current_depth + 1))
            .collect();

        for sub in sub_results {
            results.extend(sub);
        }
    } else {
        for subdir in subdirs {
            results.extend(scan_directory_parallel(
                &subdir,
                max_depth,
                current_depth + 1,
            ));
        }
    }

    results
}

fn should_skip_path(path: &Path) -> bool {
    if let Some(name) = path.file_name() {
        let name_str = name.to_string_lossy();

        #[cfg(target_os = "windows")]
        {
            if name_str.starts_with("$Recycle")
                || name_str == "System Volume Information"
                || name_str == "Windows.old"
                || name_str == "ProgramData"
                || name_str == "hiberfil.sys"
                || name_str == "pagefile.sys"
                || name_str == "DumpStack.log.tmp"
                || name_str == "Config.Msi"
            {
                return true;
            }
        }

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            if name_str.starts_with('.')
                || name_str == "proc"
                || name_str == "sys"
                || name_str == "dev"
                || name_str == "run"
                || name_str == "tmp"
                || name_str == "var"
                || name_str == "snap"
                || name_str == "lost+found"
            {
                return true;
            }
        }

        #[cfg(target_os = "macos")]
        {
            if name_str == ".Spotlight-V100"
                || name_str == ".Trashes"
                || name_str == ".fseventsd"
                || name_str == ".DocumentRevisions-V100"
                || name_str == ".TemporaryItems"
                || name_str == "Network"
                || name_str == "Volumes"
            {
                return true;
            }
        }
    }

    false
}

fn cache_build_scan_percent(completed_roots: usize, total_roots: u32) -> u8 {
    let t = total_roots.max(1) as u64;
    let c = completed_roots as u64;
    if c >= t {
        55
    } else {
        u8::try_from((c * 55) / t).unwrap_or(0).min(54)
    }
}

fn build_tree_blazing_fast(scanned_paths: &[PathBuf]) -> Vec<FileEntry> {
    shared::reset_tree_build_progress(scanned_paths.len() as u64);

    #[derive(Default)]
    struct Node {
        is_dir: bool,
        children: Vec<PathBuf>,
    }

    let mut nodes: HashMap<PathBuf, Node> = HashMap::with_capacity(scanned_paths.len());

    for path in scanned_paths {
        let is_dir = path.is_dir();

        nodes.entry(path.clone()).or_insert_with(|| Node {
            is_dir,
            children: Vec::new(),
        });

        if let Some(parent) = path.parent() {
            nodes
                .entry(parent.to_path_buf())
                .or_default()
                .children
                .push(path.clone());
        }
    }

    let has_parent: HashSet<&PathBuf> = nodes.values().flat_map(|n| n.children.iter()).collect();

    let mut roots: Vec<PathBuf> = nodes
        .keys()
        .filter(|p| !has_parent.contains(p))
        .cloned()
        .collect();

    roots.sort_unstable();

    fn build_entry(path: &PathBuf, nodes: &HashMap<PathBuf, Node>) -> FileEntry {
        let node = nodes.get(path);
        let is_dir = node.map(|n| n.is_dir).unwrap_or(false);

        // from_path now populates flags automatically — no extra work needed here.
        let mut entry = FileEntry::from_path(path, is_dir);
        shared::log_cache_tree_insert(&entry.path, entry.is_dir);

        entry.children = node.and_then(|n| {
            if n.children.is_empty() {
                None
            } else {
                const PAR_CHILD_THRESHOLD: usize = 16;
                let mut v: Vec<FileEntry> = if n.children.len() >= PAR_CHILD_THRESHOLD {
                    n.children
                        .par_iter()
                        .map(|c| build_entry(c, nodes))
                        .collect()
                } else {
                    n.children.iter().map(|c| build_entry(c, nodes)).collect()
                };

                v.sort_unstable_by(|a, b| match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                });
                Some(v)
            }
        });

        entry
    }

    let mut result: Vec<FileEntry> = roots.into_iter().map(|r| build_entry(&r, &nodes)).collect();

    result.sort_unstable_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    result
}
fn write_cache_to_disk(data: &CacheData) -> Result<(), String> {
    let cache_file = cache_file_path();
    let temp_file = cache_temp_file_path();

    match File::create(&temp_file) {
        Ok(file) => {
            let writer = BufWriter::with_capacity(16 * 1024 * 1024, file);
            let mut encoder = zstd::Encoder::new(writer, 0)
                .map_err(|e| format!("Failed to init encoder: {}", e))?;

            // Multi-threaded zstd uses extra CPU threads for compression (no GPU in portable zstd).
            let zstd_workers = (num_cpus::get() as u32).clamp(1, 8);
            if let Err(e) = encoder.multithread(zstd_workers) {
                eprintln!("zstd multithread disabled ({}); continuing single-threaded", e);
            }

            bincode::serialize_into(&mut encoder, data)
                .map_err(|e| format!("Failed to serialize cache: {}", e))?;

            let writer = encoder
                .finish()
                .map_err(|e| format!("Failed to finalize compression: {}", e))?;

            let mut writer = writer;
            writer
                .flush()
                .map_err(|e| format!("Failed to flush cache file: {}", e))?;

            drop(writer);

            match fs::metadata(&temp_file) {
                Ok(meta) => {
                    if meta.len() == 0 {
                        let _ = fs::remove_file(&temp_file);
                        return Err("Cache file is empty after write".to_string());
                    }
                }
                Err(e) => {
                    let _ = fs::remove_file(&temp_file);
                    return Err(format!("Failed to verify temp cache file: {}", e));
                }
            }

            fs::rename(&temp_file, &cache_file)
                .map_err(|e| format!("Failed to finalize cache file: {}", e))?;

            Ok(())
        }
        Err(e) => Err(format!("Failed to create temp cache file: {}", e)),
    }
}

fn update_global_cache(data: &CacheData) {
    if let Ok(mut guard) = GLOBAL_CACHE.lock() {
        *guard = Some(data.clone());
    }
}

pub fn scan_file_extensions_from_cache(cache: &CacheData) -> Vec<SupportedFileExtension> {
    let office_extensions: HashSet<&str> = [
        "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp", "pdf", "rtf", "csv",
    ]
    .iter()
    .copied()
    .collect();

    let dev_extensions: HashSet<&str> = [
        "rs",
        "c",
        "cpp",
        "h",
        "hpp",
        "cs",
        "java",
        "kt",
        "swift",
        "go",
        "py",
        "rb",
        "php",
        "sh",
        "bash",
        "zsh",
        "pl",
        "lua",
        "html",
        "css",
        "js",
        "jsx",
        "ts",
        "tsx",
        "vue",
        "svelte",
        "json",
        "yaml",
        "yml",
        "toml",
        "ini",
        "cfg",
        "env",
        "md",
        "markdown",
        "sql",
        "r",
        "asm",
        "dart",
        "scala",
        "clj",
        "cljs",
        "hs",
        "elm",
        "ex",
        "exs",
        "ml",
        "mli",
        "fs",
        "fsx",
        "fsi",
        "vb",
        "vbs",
        "ps1",
        "psm1",
        "bat",
        "cmd",
        "coffee",
        "less",
        "sass",
        "scss",
        "styl",
        "graphql",
        "gql",
        "proto",
        "thrift",
        "xml",
        "xsd",
        "xslt",
        "dtd",
        "lock",
        "gradle",
        "cmake",
        "make",
        "mk",
        "ninja",
        "dockerfile",
        "gitignore",
        "gitattributes",
        "gitmodules",
        "gitconfig",
        "editorconfig",
        "prettierrc",
        "eslintrc",
        "eslintignore",
        "babelrc",
        "tsconfig",
        "jsconfig",
        "tf",
        "tfvars",
        "tfstate",
        "hcl",
        "jade",
        "pug",
        "ejs",
        "hbs",
        "mustache",
    ]
    .iter()
    .copied()
    .collect();

    let mut extension_counts: HashMap<String, i32> = HashMap::new();

    fn walk_entries(
        entries: &[FileEntry],
        counts: &mut HashMap<String, i32>,
        office: &HashSet<&str>,
        dev: &HashSet<&str>,
    ) {
        for entry in entries {
            if !entry.is_dir {
                if let Some(ext) = Path::new(&entry.name).extension() {
                    if let Some(ext_str) = ext.to_str() {
                        let ext_lower = ext_str.to_lowercase();
                        *counts.entry(ext_lower).or_insert(0) += 1;
                    }
                }
            }
            if let Some(children) = &entry.children {
                walk_entries(children, counts, office, dev);
            }
        }
    }

    walk_entries(
        &cache.roots,
        &mut extension_counts,
        &office_extensions,
        &dev_extensions,
    );

    let mut result: Vec<SupportedFileExtension> = extension_counts
        .into_iter()
        .map(|(extension, count)| {
            let file_type = if office_extensions.contains(extension.as_str()) {
                "office".to_string()
            } else if dev_extensions.contains(extension.as_str()) {
                "dev".to_string()
            } else {
                "other".to_string()
            };
            SupportedFileExtension {
                extension,
                count,
                is_selected: false,
                file_type,
            }
        })
        .collect();

    result.sort_by(|a, b| {
        let rank = |file_type: &str| match file_type {
            "office" => 0,
            "dev" => 1,
            _ => 2,
        };
        let rank_diff = rank(a.file_type.as_str()).cmp(&rank(b.file_type.as_str()));
        if rank_diff != std::cmp::Ordering::Equal {
            return rank_diff;
        }
        a.extension.cmp(&b.extension)
    });

    result
}
fn save_extensions_to_settings(extensions: Vec<SupportedFileExtension>) {
    let settings_file = settings_path();

    let content = match fs::read_to_string(&settings_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Could not read settings file: {}", e);
            return;
        }
    };

    let mut settings: Settings = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Could not parse settings file: {}", e);
            return;
        }
    };

    let mut scanned_by_key: HashMap<String, SupportedFileExtension> = extensions
        .into_iter()
        .map(|ext| (ext.extension.to_lowercase(), ext))
        .collect();

    let mut merged: Vec<SupportedFileExtension> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    // Preserve previous selections while refreshing metadata from the latest scan.
    for existing in settings.deep_search.supported_file_extensions.iter() {
        let key = existing.extension.to_lowercase();
        if !seen.insert(key.clone()) {
            continue;
        }

        if let Some(mut scanned) = scanned_by_key.remove(&key) {
            scanned.is_selected = existing.is_selected;
            merged.push(scanned);
        } else {
            merged.push(existing.clone());
        }
    }

    // Add newly discovered extensions that were not in settings before.
    for (_, scanned) in scanned_by_key {
        let key = scanned.extension.to_lowercase();
        if seen.insert(key) {
            merged.push(scanned);
        }
    }

    merged.sort_by(|a, b| {
        let rank = |file_type: &str| match file_type {
            "office" => 0,
            "dev" => 1,
            _ => 2,
        };
        let rank_diff = rank(a.file_type.as_str()).cmp(&rank(b.file_type.as_str()));
        if rank_diff != std::cmp::Ordering::Equal {
            return rank_diff;
        }
        a.extension.cmp(&b.extension)
    });

    settings.deep_search.supported_file_extensions = merged;

    match serde_json::to_string_pretty(&settings) {
        Ok(updated) => match fs::write(&settings_file, updated) {
            Ok(_) => println!("File extensions saved to settings.json"),
            Err(e) => eprintln!("Failed to write settings file: {}", e),
        },
        Err(e) => eprintln!("Failed to serialize settings: {}", e),
    }
}
use std::path::{Path, PathBuf};
use std::fs;
use super::should_skip_path::should_skip_path;

pub fn scan_directory_parallel(dir: &Path, max_depth: u32, current_depth: u32) -> Vec<PathBuf> {
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
            .iter()
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
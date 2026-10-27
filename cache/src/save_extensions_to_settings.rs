use base_path::settings_path;
use shared::Settings;
use shared::SupportedFileExtension;
use std::collections::{HashMap, HashSet};
use std::fs;
pub fn save_extensions_to_settings(extensions: Vec<SupportedFileExtension>) {
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

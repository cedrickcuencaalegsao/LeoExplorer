use base_path::{base_path, cache_path, settings_path, get_user_home};
use shared::AllDirectories;
use shared::Cache;
use shared::Clipboard;
use shared::DeepSearchSettings;
use shared::DirectoryStats;
use shared::DirectoryStatsTraversalState;
use shared::DriveInfo;
use shared::FileMonitorIgnoreList;
use shared::FileMonitorIgnoreListLinux;
use shared::FileMonitorIgnoreListMacOS;
use shared::FileMonitorIgnoreListWindows;
use shared::Settings;
use shared::SystemSettings;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Instant;
use shared::{DIRECTORY_STATS_MAX_DEPTH, DIRECTORY_STATS_MAX_DURATION, DIRECTORY_STATS_MAX_ENTRIES};

#[cfg(target_os = "linux")]
#[path = "drives_linux.rs"]
mod drives_linux;
#[cfg(target_os = "macos")]
#[path = "drives_macos.rs"]
mod drives_macos;
#[cfg(target_os = "windows")]
#[path = "drives_windows.rs"]
mod drives_windows;


/// Returns the current username (login id).
fn get_username() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERNAME")
            .ok()
            .or_else(|| std::env::var("USER").ok())
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("USER").ok()
    }
}

fn should_stop_directory_stats(started_at: Instant, state: &DirectoryStatsTraversalState) -> bool {
    state.visited_entries >= DIRECTORY_STATS_MAX_ENTRIES
        || started_at.elapsed() >= DIRECTORY_STATS_MAX_DURATION
}

fn collect_directory_stats(
    path: &Path,
    depth: usize,
    started_at: Instant,
    state: &mut DirectoryStatsTraversalState,
) -> std::io::Result<DirectoryStats> {
    let mut stats = DirectoryStats::default();
    if depth >= DIRECTORY_STATS_MAX_DEPTH || should_stop_directory_stats(started_at, state) {
        state.limit_hit = true;
        return Ok(stats);
    }

    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(err) => return Err(err),
    };

    for entry_result in entries {
        if should_stop_directory_stats(started_at, state) {
            state.limit_hit = true;
            break;
        }

        let entry = match entry_result {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        state.visited_entries += 1;

        let entry_path = entry.path();
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };

        if metadata.is_dir() {
            stats.folder_count += 1;
            if let Ok(child_stats) =
                collect_directory_stats(&entry_path, depth + 1, started_at, state)
            {
                stats.file_count += child_stats.file_count;
                stats.folder_count += child_stats.folder_count;
                stats.total_size_bytes += child_stats.total_size_bytes;
            }
        } else if metadata.is_file() {
            stats.file_count += 1;
            stats.total_size_bytes += metadata.len();
        }
    }

    Ok(stats)
}

#[cfg(target_os = "linux")]
fn linux_xdg_user_dir(key: &str) -> Option<PathBuf> {
    let config_home = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| get_user_home().map(|h| h.join(".config")))?;

    let dirs_file = config_home.join("user-dirs.dirs");
    let contents = fs::read_to_string(&dirs_file).ok()?;
    let needle = format!("XDG_{}_DIR=", key);

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix(&needle) {
            let value = rest.trim().trim_matches('"');
            let expanded = if let Some(stripped) = value
                .strip_prefix("$HOME")
                .or_else(|| value.strip_prefix("${HOME}"))
            {
                let home = get_user_home()?;
                home.join(stripped.trim_start_matches('/'))
            } else {
                PathBuf::from(value)
            };
            return expanded.exists().then_some(expanded);
        }
    }
    None
}

/// ~/Desktop (Linux: XDG DESKTOP when available)
pub fn get_desktop_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    if let Some(p) = linux_xdg_user_dir("DESKTOP") {
        return Some(p);
    }
    get_user_home().map(|h| h.join("Desktop"))
}

/// ~/Documents
pub fn get_documents_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    if let Some(p) = linux_xdg_user_dir("DOCUMENTS") {
        return Some(p);
    }
    get_user_home().map(|h| h.join("Documents"))
}

/// ~/Downloads
pub fn get_downloads_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    if let Some(p) = linux_xdg_user_dir("DOWNLOAD") {
        return Some(p);
    }
    get_user_home().map(|h| h.join("Downloads"))
}

/// ~/Music
pub fn get_music_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    if let Some(p) = linux_xdg_user_dir("MUSIC") {
        return Some(p);
    }
    get_user_home().map(|h| h.join("Music"))
}

/// ~/Pictures
pub fn get_pictures_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    if let Some(p) = linux_xdg_user_dir("PICTURES") {
        return Some(p);
    }
    get_user_home().map(|h| h.join("Pictures"))
}

/// ~/Movies on macOS; Linux uses XDG VIDEOS or ~/Videos / ~/Movies
pub fn get_movies_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        if let Some(p) = linux_xdg_user_dir("VIDEOS") {
            return Some(p);
        }
        return get_user_home().and_then(|h| {
            let videos = h.join("Videos");
            if videos.exists() {
                Some(videos)
            } else {
                Some(h.join("Movies"))
            }
        });
    }
    #[cfg(not(target_os = "linux"))]
    get_user_home().map(|h| h.join("Movies"))
}

/// /Applications on macOS; %ProgramFiles% on Windows
pub fn get_applications_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("ProgramFiles")
            .ok()
            .map(PathBuf::from)
            .filter(|p| p.exists())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let p = PathBuf::from("/Applications");
        if p.exists() { Some(p) } else { None }
    }
}

/// ~/Applications  — per-user app installs (shown in some Finder configs)
pub fn get_user_applications_path() -> Option<PathBuf> {
    get_user_home()
        .map(|h| h.join("Applications"))
        .filter(|p| p.exists())
}

/// The home folder itself (Finder shows it as "<Username>")
pub fn get_home_path() -> Option<PathBuf> {
    get_user_home()
}

/// ~/Public  — optional Finder favourite
pub fn get_public_path() -> Option<PathBuf> {
    get_user_home()
        .map(|h| h.join("Public"))
        .filter(|p| p.exists())
}

// ─────────────────────────────────────────────────────────────────────────────
// iCloud Drive paths
// macOS stores iCloud Drive data under
//   ~/Library/Mobile Documents/com~apple~CloudDocs/
// Synced Desktop and Documents live there when "Desktop & Documents" is on.
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
fn icloud_root() -> Option<PathBuf> {
    get_user_home().map(|h| {
        h.join("Library")
            .join("Mobile Documents")
            .join("com~apple~CloudDocs")
    })
}

/// iCloud Drive root (equivalent to ~/iCloud Drive in Finder)
#[cfg(target_os = "macos")]
pub fn get_icloud_drive_path() -> Option<PathBuf> {
    icloud_root().filter(|p| p.exists())
}

/// iCloud-synced Desktop (only present when "Desktop & Documents Folders" is enabled in iCloud settings)
#[cfg(target_os = "macos")]
pub fn get_icloud_desktop_path() -> Option<PathBuf> {
    icloud_root()
        .map(|r| r.join("Desktop"))
        .filter(|p| p.exists())
}

/// iCloud-synced Documents
#[cfg(target_os = "macos")]
pub fn get_icloud_documents_path() -> Option<PathBuf> {
    icloud_root()
        .map(|r| r.join("Documents"))
        .filter(|p| p.exists())
}

// ─────────────────────────────────────────────────────────────────────────────
// Volume / drive enumeration — platform-specific (see drives_*.rs)
// ─────────────────────────────────────────────────────────────────────────────

pub fn get_system_drives() -> Vec<DriveInfo> {
    #[cfg(target_os = "macos")]
    {
        drives_macos::get_system_drives()
    }
    #[cfg(target_os = "windows")]
    {
        drives_windows::get_system_drives()
    }
    #[cfg(target_os = "linux")]
    {
        drives_linux::get_system_drives()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Vec::new()
    }
}

pub fn init_settings() -> Result<(), String> {
    let settings_file = settings_path();

    if let Some(parent) = settings_file.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create settings directory: {}", e))?;
    }

    if settings_file.exists() {
        let content = fs::read_to_string(&settings_file)
            .map_err(|e| format!("Failed to read settings.json: {}", e))?;

        if serde_json::from_str::<Settings>(&content).is_err() {
            println!("settings.json is malformed — resetting to default");
            let _ = write_default_settings(&settings_file);
        } else {
            println!("settings.json is valid");
        }
    } else {
        println!("settings.json not found — creating default");
        write_default_settings(&settings_file)?;
    }
    Ok(())
}

fn write_default_settings(settings_file: &std::path::Path) -> Result<(), String> {
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let content = serde_json::to_string_pretty(&create_default_settings(cores))
        .map_err(|e| format!("Failed to serialize default settings: {}", e))?;

    fs::write(settings_file, content).map_err(|e| {
        format!(
            "Failed to write default settings to {}: {}",
            settings_file.display(),
            e
        )
    })?;

    println!("Default settings written to: {:?}", settings_file);
    Ok(())
}

pub fn default_settings() -> Settings{
    let core = get_num_cores().unwrap_or(1);

    create_default_settings(core)
}

fn create_default_settings(cores: usize) -> Settings {

    println!("number of cores:{}", cores);

    Settings {
        cache: Cache {
            num_of_cores: cores,
            thread_count_per_core: 2,
        },
        deep_search: DeepSearchSettings {
            thread_count_per_core: 2,
            stop_after_match: false,
            include_dev_files: true,
            supported_file_extensions: Vec::new(),
            drives: Vec::new(),
            folders: Vec::new(),
            order: "asc".to_string(),
        },
        system_settings: SystemSettings {
            show_hidden_files: false,
            folder_sort_by_date: true,
            folder_sort_order: "desc".to_string(),
            theme: "light".to_string(),
        },
        ignore_list: FileMonitorIgnoreList {
            windows: FileMonitorIgnoreListWindows {
                system_dirs: vec![
                    "c:\\windows\\",
                    "c:\\programdata\\",
                    "c:\\$recycle.bin\\",
                    "c:\\system volume information\\",
                    "c:\\recovery\\",
                    "c:\\program files\\windowsapps\\",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                app_data_patterns: vec![
                    "\\appdata\\local\\temp\\",
                    "\\appdata\\locallow\\temp\\",
                    "\\appdata\\local\\microsoft\\windows\\inetcache\\",
                    "\\appdata\\local\\packages\\",
                    "\\microsoft\\windows\\recent\\",
                    "\\node_modules\\",
                    "\\.git\\objects\\",
                    "\\.nuget\\packages\\",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                system_patterns: vec![
                    ".tmp",
                    ".temp",
                    ".log",
                    ".log.gz",
                    ".dll",
                    ".exe",
                    ".msi",
                    ".pdb",
                    ".cache",
                    ".lock",
                    ".db-wal",
                    ".db-shm",
                    "thumbs.db",
                    "desktop.ini",
                    ".pyc",
                    ".pyo",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                browser_patterns: vec![
                    "History",
                    "Cache",
                    "Code Cache",
                    "GPU Cache",
                    "Cookies",
                    "IndexedDB",
                    "Local Storage",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            },
            linux: FileMonitorIgnoreListLinux {
                system_dirs: vec![
                    "/proc/",
                    "/sys/",
                    "/dev/",
                    "/run/",
                    "/tmp/",
                    "/var/cache/",
                    "/var/tmp/",
                    "/usr/",
                    "/boot/",
                    "/lost+found/",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                app_data_patterns: vec![
                    "/.cache/",
                    "/.local/share/Trash/",
                    "/node_modules/",
                    "/.npm/",
                    "/.cargo/registry/",
                    "/.gradle/",
                    "/.m2/repository/",
                    "/snap/",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                system_patterns: vec![
                    ".tmp", ".temp", ".swp", ".log", ".log.gz", ".lock", ".pid", ".pyc", ".pyo",
                    ".o", ".so", ".a", ".db-wal", ".db-shm",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                browser_patterns: vec![
                    "History",
                    "Cache",
                    "Code Cache",
                    "GPU Cache",
                    "Cookies",
                    "IndexedDB",
                    "Local Storage",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            },
            macos: FileMonitorIgnoreListMacOS {
                system_dirs: vec![
                    // ── Core OS ──────────────────────────────────────────────
                    "/System/",
                    "/System Volumes/",
                    "/private/var/",
                    "/private/etc/",
                    "/private/tmp/",
                    "/usr/",
                    "/bin/",
                    "/sbin/",
                    "/cores/",
                    "/dev/",
                    // ── macOS application support & caches ───────────────────
                    "/Library/Caches/",
                    "/Library/Logs/",
                    "/Library/Application Support/CrashReporter/",
                    "/Library/Application Support/Apple/",
                    "/Library/Application Support/com.apple.",
                    "/Library/Preferences/",
                    "/Library/Saved Application State/",
                    "/Library/Containers/",
                    "/Library/Group Containers/",
                    "/Library/Keychains/",
                    "/Library/WebKit/",
                    "/Library/Assistant/",
                    "/Library/Suggestions/",
                    "/Library/Personalizations/",
                    "/Library/Biome/",
                    // ── User-level noise ─────────────────────────────────────
                    "/.Trash/",
                    "/.Spotlight-V100/",
                    "/.fseventsd/",
                    "/.DocumentRevisions-V100/",
                    "/.TemporaryItems/",
                    "/.vol/",
                    "/.hotfiles.btree",
                    // ── Time Machine & backups ───────────────────────────────
                    "/Backups.backupdb/",
                    "/.MobileBackups/",
                    // ── Xcode / developer tools ──────────────────────────────
                    "/DerivedData/",
                    "/Xcode/DerivedData/",
                    "/Library/Developer/Xcode/DerivedData/",
                    "/Library/Developer/Xcode/Archives/",
                    "/Library/Developer/Xcode/iOS Device Logs/",
                    "/Library/Developer/CoreSimulator/",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),

                app_data_patterns: vec![
                    // ── Browsers ─────────────────────────────────────────────
                    "/Library/Application Support/Google/Chrome/",
                    "/Library/Application Support/Microsoft Edge/",
                    "/Library/Application Support/Firefox/",
                    "/Library/Application Support/Safari/",
                    "/Library/Application Support/Brave Browser/",
                    "/Library/Application Support/Opera/",
                    "/Library/Application Support/Vivaldi/",
                    "/Library/Application Support/Arc/",
                    // ── Communication ────────────────────────────────────────
                    "/Library/Application Support/Slack/",
                    "/Library/Application Support/discord/",
                    "/Library/Application Support/zoom.us/",
                    "/Library/Application Support/Microsoft/Teams/",
                    "/Library/Application Support/WhatsApp/",
                    "/Library/Application Support/Telegram Desktop/",
                    // ── Developer tools ──────────────────────────────────────
                    "/Library/Application Support/Code/",
                    "/Library/Application Support/JetBrains/",
                    "/Library/Application Support/Sublime Text/",
                    // ── Media & streaming ────────────────────────────────────
                    "/Library/Application Support/Spotify/",
                    "/Library/Application Support/Steam/",
                    "/Library/Application Support/com.apple.iTunes/",
                    // ── Cloud sync metadata ──────────────────────────────────
                    "/Library/Application Support/CloudDocs/",
                    "/Library/Application Support/com.apple.sharedfilelist/",
                    "/Library/Application Support/MobileSync/",
                    // ── Package managers / build tools ───────────────────────
                    "/.npm/",
                    "/.cargo/registry/",
                    "/.gradle/",
                    "/.m2/repository/",
                    "/node_modules/",
                    "/.cocoapods/",
                    "/.pub-cache/",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),

                system_patterns: vec![
                    // Temp & swap
                    ".tmp",
                    ".temp",
                    ".swp",
                    ".swap",
                    // Logs
                    ".log",
                    ".log.gz",
                    ".log.1",
                    ".log.2",
                    // macOS metadata & resource forks
                    ".DS_Store",
                    "._", // AppleDouble resource fork prefix
                    ".localized",
                    ".metadata_never_index",
                    ".nomedia",
                    // Crash reports & diagnostics
                    ".crash",
                    ".ips", // iOS/macOS crash report
                    ".diag",
                    ".spin", // hang reports
                    // Compiled / binary artifacts
                    ".o",
                    ".a",
                    ".dylib",
                    ".pyc",
                    ".pyo",
                    ".class",
                    // Lock & state files
                    ".lock",
                    ".pid",
                    "-journal",
                    ".db-wal",
                    ".db-shm",
                    // Partial downloads
                    ".crdownload",
                    ".part",
                    ".partial",
                    ".download",
                    // Build artifacts
                    ".out",
                    ".dSYM",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),

                browser_patterns: vec![
                    "History",
                    "History-journal",
                    "Cookies",
                    "Cookies.bak",
                    "Cache",
                    "Code Cache",
                    "GPU Cache",
                    "Shader Cache",
                    "Media Cache",
                    "Preferences",
                    "Secure Preferences",
                    "Local Storage",
                    "Session Storage",
                    "Service Worker",
                    "IndexedDB",
                    "Web Data",
                    "Login Data",
                    "Network Persistent State",
                    "QuotaManager",
                    "Reporting and NEL",
                    "Trust Tokens",
                    "Browsing Topics State",
                    "Shared Proto Db",
                    "Optimization Guide Hint Cache Data",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            },
        },
        clipboard: Clipboard::default(),
        pin_folders: Vec::new(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Cache helpers (unchanged from original, kept for completeness)
// ─────────────────────────────────────────────────────────────────────────────

pub fn cleanup_cache_on_exit() -> Result<(), String> {
    let base_dir = base_path();

    if !base_dir.exists() {
        println!("Base directory does not exist, nothing to clean up.");
        return Ok(());
    }

    let leocache_path = base_dir.join("leocache.bin.zst");
    if leocache_path.exists() {
        fs::remove_file(&leocache_path)
            .map_err(|e| format!("Failed to delete leocache.bin.zst: {}", e))?;
        println!("Deleted leocache.bin.zst");
    }

    let cache_dir = cache_path();
    if cache_dir.exists() {
        let mut deleted = 0usize;
        let mut errors = 0usize;

        for entry in fs::read_dir(&cache_dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.ends_with("settings.json") {
                continue;
            }
            let result = if path.is_dir() {
                fs::remove_dir_all(&path)
            } else {
                fs::remove_file(&path)
            };
            match result {
                Ok(_) => deleted += 1,
                Err(e) => {
                    eprintln!("Failed to delete {:?}: {}", path, e);
                    errors += 1;
                }
            }
        }

        println!("Cache cleanup: {} deleted, {} errors", deleted, errors);
        if errors > 0 {
            return Err(format!("Failed to delete {} cache items", errors));
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tauri commands
// ─────────────────────────────────────────────────────────────────────────────

// #[tauri::command]
pub fn get_num_cores() -> Result<usize, String> {
    let cores = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    println!("CPU logical cores: {}", cores);
    Ok(cores)
}

// #[tauri::command]
pub fn get_directory_stats(path: String) -> Result<DirectoryStats, String> {
    let started_at = Instant::now();
    let path_buf = PathBuf::from(path);
    if !path_buf.exists() {
        return Err(format!("Path does not exist: {}", path_buf.display()));
    }
    if !path_buf.is_dir() {
        return Err(format!("Path is not a directory: {}", path_buf.display()));
    }

    let mut traversal_state = DirectoryStatsTraversalState::default();
    let stats =
        collect_directory_stats(&path_buf, 0, started_at, &mut traversal_state).map_err(|e| {
            format!(
                "Failed to collect directory stats for {}: {}",
                path_buf.display(),
                e
            )
        })?;

    let elapsed_ms = started_at.elapsed().as_millis();
    println!(
        "[system][perf] get_directory_stats path={} elapsed={}ms visited_entries={} limit_hit={}",
        path_buf.display(),
        elapsed_ms,
        traversal_state.visited_entries,
        traversal_state.limit_hit
    );

    Ok(stats)
}

/// Returns every directory Finder shows in its sidebar, split into three groups:
///   special_dirs  — Favourites (Desktop, Documents, Downloads, …, Home, Applications)
///   drives        — Locations  (Macintosh HD + /Volumes/*)
///   icloud_dirs   — iCloud Drive root + Desktop/Documents when iCloud sync is on
// #[tauri::command]
pub fn get_all_dirs_and_drives() -> AllDirectories {
    let mut special_dirs: HashMap<String, PathBuf> = HashMap::new();

    // ── Finder Favourites order ───────────────────────────────────────────────
    // 1. AirDrop  — no filesystem path, intentionally skipped
    // 2. Recents  — virtual, skipped
    // 3. Home folder
    // Insert under stable key "Home" (frontend uses this) AND the actual
    // username (matches what Finder shows as the sidebar label).
    if let Some(p) = get_home_path() {
        special_dirs.insert("Home".to_string(), p.clone());
        if let Some(username) = get_username() {
            special_dirs.insert(username, p);
        }
    }
    // 4. Desktop
    if let Some(p) = get_desktop_path() {
        special_dirs.insert("Desktop".to_string(), p);
    }
    // 5. Documents
    if let Some(p) = get_documents_path() {
        special_dirs.insert("Documents".to_string(), p);
    }
    // 6. Downloads
    if let Some(p) = get_downloads_path() {
        special_dirs.insert("Downloads".to_string(), p);
    }
    // 7. Movies
    if let Some(p) = get_movies_path() {
        special_dirs.insert("Movies".to_string(), p);
    }
    // 8. Music
    if let Some(p) = get_music_path() {
        special_dirs.insert("Music".to_string(), p);
    }
    // 9. Pictures
    if let Some(p) = get_pictures_path() {
        special_dirs.insert("Pictures".to_string(), p);
    }
    // 10. Applications
    if let Some(p) = get_applications_path() {
        special_dirs.insert("Applications".to_string(), p);
    }
    if let Some(p) = get_user_applications_path() {
        special_dirs.insert("User Applications".to_string(), p);
    }
    // 11. Public
    if let Some(p) = get_public_path() {
        special_dirs.insert("Public".to_string(), p);
    }

    // ── iCloud (macOS only) ───────────────────────────────────────────────────
    #[cfg(target_os = "macos")]
    let mut icloud_dirs: HashMap<String, PathBuf> = HashMap::new();
    #[cfg(not(target_os = "macos"))]
    let icloud_dirs: HashMap<String, PathBuf> = HashMap::new();
    #[cfg(target_os = "macos")]
    {
        if let Some(p) = get_icloud_drive_path() {
            icloud_dirs.insert("iCloud Drive".to_string(), p);
        }
        if let Some(p) = get_icloud_desktop_path() {
            icloud_dirs.insert("iCloud Desktop".to_string(), p);
        }
        if let Some(p) = get_icloud_documents_path() {
            icloud_dirs.insert("iCloud Documents".to_string(), p);
        }
    }

    // ── Locations ────────────────────────────────────────────────────────────
    let drives = get_system_drives();

    AllDirectories {
        special_dirs,
        drives,
        icloud_dirs,
    }
}

// #[tauri::command]
pub fn delete_specific_cache_files() -> Result<(), String> {
    let cache_dir = cache_path();
    if !cache_dir.exists() {
        return Ok(());
    }

    for name in &[
        "leo_cache.db",
        "leo_cache.json",
        "search_cache.db",
        "search_cache.json",
        "temp_cache.db",
        "temp_cache.json",
        "index.db",
        "index.json",
    ] {
        let p = cache_dir.join(name);
        if p.exists() {
            if let Err(e) = fs::remove_file(&p) {
                eprintln!("Failed to delete {}: {}", name, e);
            }
        }
    }
    Ok(())
}

// #[tauri::command]
pub fn preserve_settings_clean_cache() -> Result<(), String> {
    let settings_file = settings_path();
    let backup = settings_file
        .exists()
        .then(|| fs::read_to_string(&settings_file).ok())
        .flatten();

    cleanup_cache_on_exit()?;

    if let Some(content) = backup {
        if let Some(parent) = settings_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to recreate settings dir: {}", e))?;
        }
        fs::write(&settings_file, content)
            .map_err(|e| format!("Failed to restore settings: {}", e))?;
        println!("Settings restored after cache cleanup");
    }
    Ok(())
}

// #[tauri::command]
pub fn get_cache_size() -> Result<u64, String> {
    let cache_dir = cache_path();
    if !cache_dir.exists() {
        return Ok(0);
    }

    fn dir_size(path: &PathBuf) -> std::io::Result<u64> {
        let mut total = 0u64;
        for entry in fs::read_dir(path)?.flatten() {
            let p = entry.path();
            total += if p.is_dir() {
                dir_size(&p)?
            } else {
                entry.metadata()?.len()
            };
        }
        Ok(total)
    }

    dir_size(&cache_dir).map_err(|e| format!("Failed to calculate cache size: {}", e))
}

// #[tauri::command]
// pub fn update_deep_search_settings(settings: DeepSearchSettingsPayload) -> Result<(), String> {
//     crate::repository::deepsearch::deepsearchsettings::save_deep_search_settings(settings)
// }

// #[tauri::command]
// pub fn get_supported_file_extensions() -> Result<Vec<SupportedFileExtension>, String> {
//     crate::repository::deepsearch::deepsearchsettings::get_supported_file_extensions()
// }

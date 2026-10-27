use std::path::PathBuf;

pub fn base_path() -> PathBuf {
    let base_path = if let Ok(home) = std::env::var("USERPROFILE") {
        // Windows
        PathBuf::from(home)
    } else if let Ok(home) = std::env::var("HOME") {
        // Unix-like
        PathBuf::from(home)
    } else {
        PathBuf::from(".")
    };

    base_path.join(".leo_explorer")
}

pub fn cache_path() -> PathBuf {
    base_path().join("cache")
}

pub fn settings_path() -> PathBuf {
    base_path().join("settings").join("settings.json")
}

pub fn find_cache() -> bool {
    let base_path = base_path();
    if !base_path.exists(){
        return false;
    }

    base_path.join("leocache.zst").exists()
}

pub fn get_user_home() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE")
            .ok()
            .map(PathBuf::from)
            .or_else(|| std::env::var("HOME").ok().map(PathBuf::from))
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}
use serde::Serialize;

/// Progress payload for the `cache-build-progress` Tauri event during `create_cache`.
/// Percent stays in 0–99 until the UI clears on `cache_completed` / `cache_ready` / `cache_error`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheBuildProgress {
    pub phase: String,
    pub percent: u8,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drives_completed: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_drives: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths_found: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
}

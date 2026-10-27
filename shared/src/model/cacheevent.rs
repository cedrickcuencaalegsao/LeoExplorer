use std::sync::Arc;
use super::CacheBuildProgress;

#[derive(Debug, Clone)]
pub enum CacheEvent {
    Ready(String),
    Creating(String),
    Started(String),
    Progress(String),
    BuildProgress(CacheBuildProgress),
    Completed(String),
    ExtensionsReady(String),
    Error(String),
}

pub type Reporter = Arc<dyn Fn(CacheEvent) + Send + Sync>;

pub fn emit_cache_build_progress(report: &Reporter, progress: CacheBuildProgress) {
    report(CacheEvent::BuildProgress(progress));
}
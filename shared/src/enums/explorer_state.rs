use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExplorerState {
    Pinned,
    Shared,
    OfflineAvailable,
    Shortcut,
    MountPoint,
}
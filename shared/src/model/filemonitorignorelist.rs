use crate::model::filemonitorignorelistlinux::FileMonitorIgnoreListLinux;
use crate::model::filemonitorignorelistmacos::FileMonitorIgnoreListMacOS;
use crate::model::filemonitorignorelistwindows::FileMonitorIgnoreListWindows;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct FileMonitorIgnoreList {
    pub windows: FileMonitorIgnoreListWindows,
    pub linux: FileMonitorIgnoreListLinux,
    pub macos: FileMonitorIgnoreListMacOS,
}

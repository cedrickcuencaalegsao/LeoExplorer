use crate::model::cache::Cache;
use crate::model::deepsearchsettings::DeepSearchSettings;
use crate::model::filemonitorignorelist::FileMonitorIgnoreList;
use crate::model::SystemSettings;
use crate::model::clipboard::Clipboard;
use crate::model::pinfolder::PinFolder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub cache: Cache,
    pub deep_search: DeepSearchSettings,
    pub system_settings: SystemSettings,
    pub ignore_list: FileMonitorIgnoreList,
    pub clipboard: Clipboard,
    pub pin_folders: Vec<PinFolder>,
}

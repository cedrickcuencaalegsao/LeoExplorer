use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(dead_code)]
pub struct TrashEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub entry_type: String,
    pub file_type: Option<String>,
    pub size: Option<u64>,
    pub date_modified: Option<String>,
    pub date_created: Option<String>,
    pub original_path: Option<String>,
    pub date_deleted: Option<String>,
}

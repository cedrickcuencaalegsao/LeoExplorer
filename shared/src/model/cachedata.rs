use crate::model::fileentry::FileEntry;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CacheData {
    pub roots: Vec<FileEntry>,
}

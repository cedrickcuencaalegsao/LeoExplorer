use super::FileEntry;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InMemorCache {
    pub root: HashMap<PathBuf, FileEntry>,
}

impl InMemorCache {
    pub fn new() -> Self {
        Self {
            root: HashMap::new(),
        }
    }
}

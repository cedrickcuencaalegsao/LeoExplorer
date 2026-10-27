use std::{collections::HashMap, path::PathBuf};
use serde::{Deserialize, Serialize};
use super::DriveInfo;

#[derive(Debug, Serialize, Deserialize)]
pub struct AllDirectories{
    pub special_dirs: HashMap<String, PathBuf>,
    pub drives: Vec<DriveInfo>,
    pub icloud_dirs: HashMap<String, PathBuf>,
}
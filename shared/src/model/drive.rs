use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum DriveType {
    Unknown,
    NoRootDir,
    Removable,
    Fixed,
    Remote,
    CDRom,
    RamDisk,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DriveInfo {
    pub path: String,
    pub letter: Option<char>,
    pub name: String,
    pub drive_type: DriveType,
    pub file_system: Option<String>,
    pub total_space: u64,
    pub free_space: u64,
    pub used_space: u64,
    pub total_space_gb: f64,
    pub free_space_gb: f64,
    pub used_space_gb: f64,
    pub usage_percent: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct DriveList {
    pub drives: Vec<DriveInfo>,
}

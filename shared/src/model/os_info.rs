use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OsInfo {
    pub os: String,
    pub name: String,
    pub version: String,
    pub arch: String,
}
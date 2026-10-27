use serde::{Deserialize, Serialize};
use crate::enums::Flag;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinFolder {
    pub path: String,
    pub name: String,
    pub flags: Vec<Flag>,
    #[serde(default)]
    pub created_at: i64,
}

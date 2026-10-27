use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub path: String,
    pub operation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Clipboard {
    pub items: Vec<ClipboardItem>,
}

impl Default for Clipboard {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}
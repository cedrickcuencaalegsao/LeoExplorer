use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemSettings {
    #[serde(default)]
    pub show_hidden_files: bool,
    #[serde(default = "default_folder_sort_by_date")]
    pub folder_sort_by_date: bool,
    #[serde(default = "default_folder_sort_order")]
    pub folder_sort_order: String,
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_folder_sort_by_date() -> bool {
    true
}

fn default_folder_sort_order() -> String {
    "desc".to_string()
}

fn default_theme() -> String {
    "light".to_string()
}

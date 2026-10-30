use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub hidden: bool,
    pub permission_required: bool,
    pub modified: Option<SystemTime>,
}

impl Entry {
    pub fn new(name: impl Into<String>, is_dir: bool) -> Self {
        Self {
            name: name.into(),
            path: PathBuf::new(),
            is_dir,
            hidden: false,
            permission_required: false,
            modified: None,
        }
    }

    pub fn path(mut self, path: PathBuf) -> Self {
        self.path = path;
        self
    }

    pub fn modified(mut self, modified: Option<SystemTime>) -> Self {
        self.modified = modified;
        self
    }

    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    pub fn locked(mut self) -> Self {
        self.permission_required = true;
        self
    }
}

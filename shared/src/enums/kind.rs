use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Kind {
    Image,
    Video,
    Audio,
    Document,
    Code,
    Archive,
}

impl Kind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "image" => Some(Kind::Image),
            "video" => Some(Kind::Video),
            "audio" => Some(Kind::Audio),
            "document" => Some(Kind::Document),
            "code" => Some(Kind::Code),
            "archive" => Some(Kind::Archive),
            _ => None,
        }
    }

    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Kind::Image => &[
                "jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico", "tiff", "tif",
                "heic", "heif", "raw", "cr2", "nef", "arw", "dng", "avif",
            ],
            Kind::Video => &[
                "mp4", "mov", "avi", "mkv", "wmv", "flv", "webm", "m4v", "mpeg", "mpg", "3gp",
                "ts",
            ],
            Kind::Audio => &[
                "mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "opus", "aiff", "aif",
            ],
            Kind::Document => &[
                "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp", "rtf",
                "txt", "csv", "md",
            ],
            Kind::Code => &[
                "rs", "py", "js", "ts", "jsx", "tsx", "html", "css", "json", "toml", "yaml",
                "yml", "sh", "bash", "zsh", "c", "cpp", "h", "hpp", "java", "kt", "swift",
                "go", "rb", "php", "lua", "dart", "sql",
            ],
            Kind::Archive => &[
                "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "dmg", "iso", "tgz",
            ],
        }
    }
}

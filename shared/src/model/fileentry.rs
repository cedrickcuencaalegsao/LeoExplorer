use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use crate::enums::Flag;
use crate::enums::FsAttribute;
use crate::enums::Permissions;
use crate::enums::ExplorerState;
use crate::enums::CloudStatus;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub entry_type: String,        // file or folder
    pub file_type: Option<String>, // e.g., txt, jpg
    pub size: Option<u64>,
    pub flag: Vec<Flag>,           // Flags for the file entry
    pub date_modified: Option<String>,
    pub date_created: Option<String>,
    pub children: Option<Vec<FileEntry>>,
}

impl FileEntry {
    pub fn from_path(path: &Path, is_dir: bool) -> Self {
        let path_str = Self::normalize_path(path);

        // Get file name
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                path.iter()
                    .last()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| path_str.clone())
            });

        // Set entry type
        let entry_type = if is_dir {
            "folder".to_string()
        } else {
            "file".to_string()
        };

        // Get file extension if it's a file
        let file_type = if !is_dir {
            path.extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase())
        } else {
            None
        };

        // Get metadata once and reuse
        let metadata_result = fs::metadata(path).ok();

        let size = if !is_dir {
            metadata_result.as_ref().map(|m| m.len())
        } else {
            None
        };

        let date_modified = metadata_result.as_ref().and_then(|m| {
            m.modified()
                .ok()
                .and_then(|time| Self::format_system_time(time))
        });

        let date_created = metadata_result.as_ref().and_then(|m| {
            m.created()
                .ok()
                .and_then(|time| Self::format_system_time(time))
        });

        // Collect all flags from OS metadata
        let flag = Self::collect_flags(path, is_dir, &metadata_result);

        FileEntry {
            name,
            path: path_str,
            is_dir,
            entry_type,
            file_type,
            size,
            flag,
            date_modified,
            date_created,
            children: None,
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Flag collection entry point
    // ─────────────────────────────────────────────────────────────────────────

    fn collect_flags(
        path: &Path,
        is_dir: bool,
        metadata: &Option<fs::Metadata>,
    ) -> Vec<Flag> {
        let mut flags: Vec<Flag> = Vec::new();

        Self::collect_attributes(path, metadata, &mut flags);
        Self::collect_permissions(path, metadata, &mut flags);
        Self::collect_explorer_state(path, is_dir, &mut flags);
        Self::collect_cloud_status(path, &mut flags);

        flags
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Attributes
    // ─────────────────────────────────────────────────────────────────────────

    fn collect_attributes(
        _path: &Path,
        metadata: &Option<fs::Metadata>,
        flags: &mut Vec<Flag>,
    ) {
        #[cfg(target_os = "windows")]
        Self::collect_windows_attributes(metadata, flags);

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        Self::collect_unix_attributes(_path, metadata, flags);
    }

    #[cfg(target_os = "windows")]
    fn collect_windows_attributes(metadata: &Option<fs::Metadata>, flags: &mut Vec<Flag>) {
        use std::os::windows::fs::MetadataExt;

        const FILE_ATTRIBUTE_READONLY: u32    = 0x0000_0001;
        const FILE_ATTRIBUTE_HIDDEN: u32      = 0x0000_0002;
        const FILE_ATTRIBUTE_SYSTEM: u32      = 0x0000_0004;
        const FILE_ATTRIBUTE_ARCHIVE: u32     = 0x0000_0020;
        const FILE_ATTRIBUTE_COMPRESSED: u32  = 0x0000_0800;
        const FILE_ATTRIBUTE_ENCRYPTED: u32   = 0x0000_4000;
        // When this bit is SET it means NOT indexed — so indexed = bit is clear
        const FILE_ATTRIBUTE_NOT_INDEXED: u32 = 0x0000_2000;

        if let Some(meta) = metadata {
            let attrs = meta.file_attributes();

            if attrs & FILE_ATTRIBUTE_HIDDEN != 0 {
                flags.push(Flag::Attribute(FsAttribute::Hidden));
            }
            if attrs & FILE_ATTRIBUTE_READONLY != 0 {
                flags.push(Flag::Attribute(FsAttribute::ReadOnly));
            }
            if attrs & FILE_ATTRIBUTE_SYSTEM != 0 {
                flags.push(Flag::Attribute(FsAttribute::System));
            }
            if attrs & FILE_ATTRIBUTE_ARCHIVE != 0 {
                flags.push(Flag::Attribute(FsAttribute::Archive));
            }
            if attrs & FILE_ATTRIBUTE_NOT_INDEXED == 0 {
                // Bit is clear → indexing is enabled
                flags.push(Flag::Attribute(FsAttribute::Indexed));
            }
            if attrs & FILE_ATTRIBUTE_COMPRESSED != 0 {
                flags.push(Flag::Attribute(FsAttribute::Compressed));
            }
            if attrs & FILE_ATTRIBUTE_ENCRYPTED != 0 {
                flags.push(Flag::Attribute(FsAttribute::Encrypted));
            }
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn collect_unix_attributes(
        path: &Path,
        metadata: &Option<fs::Metadata>,
        flags: &mut Vec<Flag>,
    ) {
        use std::os::unix::fs::MetadataExt;

        // Hidden: dotfile convention (Linux + macOS).
        // macOS additionally exposes a BSD UF_HIDDEN flag (set by `chflags hidden`
        // and by our hide_file/hide_folder commands) that is independent of the
        // filename, so OR it in for non-dotfiles too.
        let mut hidden = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false);

        #[cfg(target_os = "macos")]
        if !hidden {
            hidden = Self::macos_has_uf_hidden(path);
        }

        if hidden {
            flags.push(Flag::Attribute(FsAttribute::Hidden));
        }

        // ReadOnly: no write bits set for anyone
        if let Some(meta) = metadata {
            let mode = meta.mode();
            if mode & 0o222 == 0 {
                flags.push(Flag::Attribute(FsAttribute::ReadOnly));
            }
        }

        #[cfg(target_os = "macos")]
        Self::collect_macos_xattrs(path, flags);

        #[cfg(target_os = "linux")]
        Self::collect_linux_iflags(path, flags);
    }

    #[cfg(target_os = "macos")]
    fn macos_has_uf_hidden(path: &Path) -> bool {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;

        const UF_HIDDEN: u32 = 0x0000_8000;

        let c_path = match CString::new(path.as_os_str().as_bytes()) {
            Ok(p) => p,
            Err(_) => return false,
        };

        unsafe {
            let mut st: libc::stat = std::mem::zeroed();
            if libc::stat(c_path.as_ptr(), &mut st as *mut libc::stat) != 0 {
                return false;
            }
            (st.st_flags as u32) & UF_HIDDEN != 0
        }
    }

    #[cfg(target_os = "macos")]
    fn collect_macos_xattrs(path: &Path, flags: &mut Vec<Flag>) {
        use std::process::Command;

        let path_str = path.to_string_lossy().to_string();

        // BSD flags: schg/uchg = immutable, uappnd/sappnd = append-only
        if let Ok(output) = Command::new("ls")
            .args(["-lOd", &path_str])
            .output()
        {
            let out = String::from_utf8_lossy(&output.stdout);
            if out.contains("schg") || out.contains("uchg") {
                flags.push(Flag::Attribute(FsAttribute::Immutable));
            }
            if out.contains("uappnd") || out.contains("sappnd") {
                flags.push(Flag::Attribute(FsAttribute::AppendOnly));
            }
        }

        // Extended attributes: encryption hint
        if let Ok(output) = Command::new("xattr")
            .args(["-l", &path_str])
            .output()
        {
            let out = String::from_utf8_lossy(&output.stdout);
            if out.contains("com.apple.cprotect") {
                flags.push(Flag::Attribute(FsAttribute::Encrypted));
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn collect_linux_iflags(path: &Path, flags: &mut Vec<Flag>) {
        use std::fs::File;
        use std::os::unix::io::AsRawFd;

        // ioctl FS_IOC_GETFLAGS
        const FS_IOC_GETFLAGS: u64 = 0x8004_6601;
        const FS_IMMUTABLE_FL: u32 = 0x0000_0010;
        const FS_APPEND_FL: u32    = 0x0000_0020;
        const FS_COMPR_FL: u32     = 0x0000_0004;
        const FS_ENCRYPT_FL: u32   = 0x0000_0800;

        if let Ok(file) = File::open(path) {
            let fd = file.as_raw_fd();
            let mut iflags: u32 = 0;
            let ret = unsafe { libc::ioctl(fd, FS_IOC_GETFLAGS, &mut iflags as *mut u32) };

            if ret == 0 {
                if iflags & FS_IMMUTABLE_FL != 0 {
                    flags.push(Flag::Attribute(FsAttribute::Immutable));
                }
                if iflags & FS_APPEND_FL != 0 {
                    flags.push(Flag::Attribute(FsAttribute::AppendOnly));
                }
                if iflags & FS_COMPR_FL != 0 {
                    flags.push(Flag::Attribute(FsAttribute::Compressed));
                }
                if iflags & FS_ENCRYPT_FL != 0 {
                    flags.push(Flag::Attribute(FsAttribute::Encrypted));
                }
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Permissions
    // ─────────────────────────────────────────────────────────────────────────

    fn collect_permissions(
        _path: &Path,
        metadata: &Option<fs::Metadata>,
        flags: &mut Vec<Flag>,
    ) {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::fs::MetadataExt;
            const FILE_ATTRIBUTE_READONLY: u32 = 0x0000_0001;

            if let Some(meta) = metadata {
                let attrs = meta.file_attributes();
                // All accessible files on Windows are at minimum readable
                flags.push(Flag::Permission(Permissions::Read));
                if attrs & FILE_ATTRIBUTE_READONLY == 0 {
                    flags.push(Flag::Permission(Permissions::Write));
                    flags.push(Flag::Permission(Permissions::FullControl));
                }
            }
        }

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            use std::os::unix::fs::MetadataExt;

            if let Some(meta) = metadata {
                let mode = meta.mode();

                let owner_read    = 0o400u32;
                let owner_write   = 0o200u32;
                let owner_execute = 0o100u32;
                let all_rwx       = owner_read | owner_write | owner_execute;

                if mode & owner_read != 0 {
                    flags.push(Flag::Permission(Permissions::Read));
                }
                if mode & owner_write != 0 {
                    flags.push(Flag::Permission(Permissions::Write));
                }
                if mode & owner_execute != 0 {
                    flags.push(Flag::Permission(Permissions::Execute));
                }
                if mode & all_rwx == all_rwx {
                    flags.push(Flag::Permission(Permissions::FullControl));
                }
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Explorer state
    // ─────────────────────────────────────────────────────────────────────────

    fn collect_explorer_state(path: &Path, is_dir: bool, flags: &mut Vec<Flag>) {
        // Symlink → Shortcut (all platforms)
        if fs::symlink_metadata(path)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
        {
            flags.push(Flag::Explorer(ExplorerState::Shortcut));
        }

        // Mount point detection
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        if is_dir {
            Self::detect_unix_mount_point(path, flags);
        }

        #[cfg(target_os = "windows")]
        if is_dir {
            Self::detect_windows_mount_point(path, flags);
        }

        // Shared: group/other write bit on Linux directories
        #[cfg(target_os = "linux")]
        if is_dir {
            use std::os::unix::fs::MetadataExt;
            if let Ok(meta) = fs::metadata(path) {
                let mode = meta.mode();
                if mode & 0o020 != 0 || mode & 0o002 != 0 {
                    flags.push(Flag::Explorer(ExplorerState::Shared));
                }
            }
        }

        // Windows: Quick Access pinned folders via Desktop.ini
        #[cfg(target_os = "windows")]
        if is_dir {
            let desktop_ini = path.join("desktop.ini");
            if let Ok(content) = fs::read_to_string(&desktop_ini) {
                if content.contains("PinToNameSpaceTree") {
                    flags.push(Flag::Explorer(ExplorerState::Pinned));
                }
            }
        }

        // macOS: iCloud offline-available marker
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let path_str = path.to_string_lossy().to_string();
            if let Ok(output) = Command::new("xattr").args(["-l", &path_str]).output() {
                let out = String::from_utf8_lossy(&output.stdout);
                if out.contains("com.apple.ubiquity-inode-identifier")
                    && !out.contains("com.apple.icloud.itemName")
                {
                    flags.push(Flag::Explorer(ExplorerState::OfflineAvailable));
                }
            }
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn detect_unix_mount_point(path: &Path, flags: &mut Vec<Flag>) {
        use std::os::unix::fs::MetadataExt;

        // Device ID differs from parent → this is a mount point
        if let (Ok(self_meta), Some(parent)) = (fs::metadata(path), path.parent()) {
            if let Ok(parent_meta) = fs::metadata(parent) {
                if self_meta.dev() != parent_meta.dev() {
                    flags.push(Flag::Explorer(ExplorerState::MountPoint));
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn detect_windows_mount_point(path: &Path, flags: &mut Vec<Flag>) {
        use std::os::windows::fs::MetadataExt;
        // Reparse point = volume mount point or junction
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;

        if let Ok(meta) = fs::metadata(path) {
            if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                flags.push(Flag::Explorer(ExplorerState::MountPoint));
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Cloud status
    // ─────────────────────────────────────────────────────────────────────────

    fn collect_cloud_status(_path: &Path, flags: &mut Vec<Flag>) {
        #[cfg(target_os = "windows")]
        Self::collect_cloud_status_windows(_path, flags);

        #[cfg(target_os = "macos")]
        Self::collect_cloud_status_macos(_path, flags);

        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        flags.push(Flag::Cloud(CloudStatus::Local));
    }

    #[cfg(target_os = "windows")]
    fn collect_cloud_status_windows(path: &Path, flags: &mut Vec<Flag>) {
        use std::os::windows::fs::MetadataExt;

        // OneDrive cloud attribute bits
        const RECALL_ON_DATA_ACCESS: u32 = 0x0040_0000;
        const RECALL_ON_OPEN: u32        = 0x0004_0000;
        const PINNED: u32                = 0x0008_0000; // always kept local
        const UNPINNED: u32              = 0x0010_0000; // online-only preference

        if let Ok(meta) = fs::metadata(path) {
            let attrs = meta.file_attributes();

            if attrs & RECALL_ON_DATA_ACCESS != 0 || attrs & RECALL_ON_OPEN != 0 {
                flags.push(Flag::Cloud(CloudStatus::OnlineOnly));
            } else if attrs & PINNED != 0 {
                flags.push(Flag::Cloud(CloudStatus::Local));
            } else if attrs & UNPINNED != 0 {
                flags.push(Flag::Cloud(CloudStatus::OnlineOnly));
            } else {
                flags.push(Flag::Cloud(CloudStatus::Local));
            }
        } else {
            flags.push(Flag::Cloud(CloudStatus::Local));
        }
    }

    #[cfg(target_os = "macos")]
    fn collect_cloud_status_macos(path: &Path, flags: &mut Vec<Flag>) {
        use std::process::Command;

        let path_str = path.to_string_lossy().to_string();

        if let Ok(output) = Command::new("xattr").args(["-l", &path_str]).output() {
            let out = String::from_utf8_lossy(&output.stdout);

            if out.contains("com.apple.icloud.itemName") {
                // Dataless placeholder — content lives online only
                flags.push(Flag::Cloud(CloudStatus::OnlineOnly));
            } else if out.contains("com.apple.ubiquity-inode-identifier") {
                // iCloud item that is locally present
                flags.push(Flag::Cloud(CloudStatus::Local));
            } else {
                flags.push(Flag::Cloud(CloudStatus::Local));
            }
        } else {
            flags.push(Flag::Cloud(CloudStatus::Local));
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Helpers
    // ─────────────────────────────────────────────────────────────────────────

    pub fn format_system_time(time: SystemTime) -> Option<String> {
        use chrono::{DateTime, Local};

        match time.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => {
                let datetime = DateTime::<Local>::from(SystemTime::UNIX_EPOCH + duration);
                Some(datetime.format("%Y-%m-%d %H:%M:%S").to_string())
            }
            Err(_) => None,
        }
    }

    pub fn normalize_path(path: &Path) -> String {
        let path_str = path.to_string_lossy().to_string();

        if cfg!(windows) {
            path_str.replace("\\", "/")
        } else {
            path_str
        }
    }

    pub fn to_pathbuf(&self) -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(self.path.replace("/", "\\"))
        } else {
            PathBuf::from(&self.path)
        }
    }

    #[allow(dead_code)]
    pub fn absolute_path(&self) -> std::io::Result<PathBuf> {
        let path = self.to_pathbuf();
        path.canonicalize()
    }

    #[allow(dead_code)]
    pub fn exists(&self) -> bool {
        self.to_pathbuf().exists()
    }

    #[allow(dead_code)]
    pub fn new(
        name: String,
        path: String,
        is_dir: bool,
        file_type: Option<String>,
        size: Option<u64>,
        date_modified: Option<String>,
        date_created: Option<String>,
        children: Option<Vec<FileEntry>>,
    ) -> Self {
        let entry_type = if is_dir {
            "folder".to_string()
        } else {
            "file".to_string()
        };

        FileEntry {
            name,
            path: Self::normalize_path(&PathBuf::from(&path)),
            is_dir,
            entry_type,
            file_type,
            size,
            flag: Vec::new(), // flags not populated via manual constructor — use from_path
            date_modified,
            date_created,
            children,
        }
    }
}
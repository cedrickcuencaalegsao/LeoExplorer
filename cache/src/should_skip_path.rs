use std::path::Path;

pub fn should_skip_path(path: &Path) -> bool {
    if let Some(name) = path.file_name() {
        let name_str = name.to_string_lossy();

        #[cfg(target_os = "windows")]
        {
            if name_str.starts_with("$Recycle")
                || name_str == "System Volume Information"
                || name_str == "Windows.old"
                || name_str == "ProgramData"
                || name_str == "hiberfil.sys"
                || name_str == "pagefile.sys"
                || name_str == "DumpStack.log.tmp"
                || name_str == "Config.Msi"
            {
                return true;
            }
        }

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            if name_str.starts_with('.')
                || name_str == "proc"
                || name_str == "sys"
                || name_str == "dev"
                || name_str == "run"
                || name_str == "tmp"
                || name_str == "var"
                || name_str == "snap"
                || name_str == "lost+found"
            {
                return true;
            }
        }

        #[cfg(target_os = "macos")]
        {
            if name_str == ".Spotlight-V100"
                || name_str == ".Trashes"
                || name_str == ".fseventsd"
                || name_str == ".DocumentRevisions-V100"
                || name_str == ".TemporaryItems"
                || name_str == "Network"
                || name_str == "Volumes"
            {
                return true;
            }
        }
    }

    false
}
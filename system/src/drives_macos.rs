use shared::DriveInfo;
use shared::DriveType;
use std::collections::HashMap;
use std::process::Command;

/// Filesystem types that hold real user data on macOS.
fn is_user_filesystem(fstype: &str) -> bool {
    matches!(
        fstype,
        "apfs"
            | "hfs"
            | "hfs+"
            | "exfat"
            | "msdos"   // FAT32 USB drives
            | "ntfs"    // NTFS via macFUSE / Paragon
            | "fuseblk"
            | "fuse"
            | "nfs"
            | "nfs4"
            | "cifs"
            | "smbfs"
            | "afpfs"   // AFP network shares (legacy)
            | "webdav"
    )
}

/// Mount points Finder actually shows in the "Locations" section.
fn is_finder_location(mount: &str) -> bool {
    mount == "/" || mount.starts_with("/Volumes/")
}

fn get_macos_mount_fs_types() -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(output) = Command::new("mount").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                if let Some(on_pos) = line.find(" on ") {
                    let rest = &line[on_pos + 4..];
                    if let Some(paren_pos) = rest.find(" (") {
                        let mount_point = rest[..paren_pos].trim().to_string();
                        let options = &rest[paren_pos + 2..];
                        let fstype = options
                            .splitn(2, ',')
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        if !fstype.is_empty() && !mount_point.is_empty() {
                            map.insert(mount_point, fstype);
                        }
                    }
                }
            }
        }
    }
    map
}

fn display_name_for_mount(mount: &str) -> String {
    if mount == "/" {
        return "Macintosh HD".to_string();
    }
    mount
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(mount)
        .to_string()
}

fn classify_drive_type(mount: &str, fstype: Option<&str>) -> DriveType {
    if mount == "/" {
        return DriveType::Fixed;
    }

    if let Some(fs) = fstype {
        if matches!(fs, "nfs" | "nfs4" | "smbfs" | "cifs" | "afpfs" | "webdav") {
            return DriveType::Remote;
        }
    }

    if let Some(fs) = fstype {
        if fs == "cd9660" || fs == "udf" {
            return DriveType::CDRom;
        }
    }

    DriveType::Removable
}

pub fn get_system_drives() -> Vec<DriveInfo> {
    let mut drives = Vec::new();
    let fs_types = get_macos_mount_fs_types();

    if let Ok(output) = Command::new("df").args(["-k"]).output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();

                let (mount_start_col, avail_col, blocks_col) =
                    if parts.len() >= 9 && parts[4].ends_with('%') {
                        (8, 3, 1)
                    } else if parts.len() >= 6 {
                        (5, 3, 1)
                    } else {
                        continue;
                    };

                let mount_point = parts[mount_start_col..].join(" ");

                if !is_finder_location(&mount_point) {
                    continue;
                }

                let fstype_opt = fs_types.get(&mount_point).map(|s| s.as_str());
                if let Some(fs) = fstype_opt {
                    if !is_user_filesystem(fs) {
                        continue;
                    }
                }

                let blocks_1024 = parts[blocks_col].parse::<u64>().unwrap_or(0);
                if blocks_1024 == 0 {
                    continue;
                }

                let total_space = blocks_1024 * 1024;
                let free_space = parts[avail_col].parse::<u64>().unwrap_or(0) * 1024;
                let used_space = total_space.saturating_sub(free_space);

                let total_gb = total_space as f64 / (1024.0_f64.powi(3));
                let free_gb = free_space as f64 / (1024.0_f64.powi(3));
                let used_gb = (total_gb - free_gb).max(0.0);

                let usage_percent = if total_space > 0 {
                    (used_space as f64 / total_space as f64) * 100.0
                } else {
                    0.0
                };

                let fstype_string = fstype_opt.map(str::to_string);
                let drive_type = classify_drive_type(&mount_point, fstype_opt);
                let name = display_name_for_mount(&mount_point);

                drives.push(DriveInfo {
                    path: mount_point.clone(),
                    letter: None,
                    name,
                    drive_type,
                    file_system: fstype_string,
                    total_space,
                    free_space,
                    used_space,
                    total_space_gb: total_gb,
                    free_space_gb: free_gb,
                    used_space_gb: used_gb,
                    usage_percent,
                });
            }
        }
    }

    drives.sort_by(|a, b| {
        let rank = |d: &DriveType| match d {
            DriveType::Fixed => 0,
            DriveType::Removable => 1,
            DriveType::Remote => 2,
            _ => 3,
        };
        rank(&a.drive_type)
            .cmp(&rank(&b.drive_type))
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    drives
}

//! Logical drives via Win32 (letters, labels, usage).

use crate::model::drivelist::{DriveInfo, DriveType};
use windows::core::PCWSTR;
use windows::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW,
};

const DRIVE_UNKNOWN: u32 = 0;
const DRIVE_NO_ROOT_DIR: u32 = 1;
const DRIVE_REMOVABLE: u32 = 2;
const DRIVE_FIXED: u32 = 3;
const DRIVE_REMOTE: u32 = 4;
const DRIVE_CDROM: u32 = 5;
const DRIVE_RAMDISK: u32 = 6;

fn wide_to_string_lossy(w: &[u16]) -> String {
    let end = w.iter().position(|&c| c == 0).unwrap_or(w.len());
    String::from_utf16_lossy(&w[..end])
}

fn map_win_drive_type(t: u32) -> DriveType {
    match t {
        DRIVE_REMOVABLE => DriveType::Removable,
        DRIVE_FIXED => DriveType::Fixed,
        DRIVE_REMOTE => DriveType::Remote,
        DRIVE_CDROM => DriveType::CDRom,
        DRIVE_RAMDISK => DriveType::RamDisk,
        DRIVE_NO_ROOT_DIR => DriveType::NoRootDir,
        _ => DriveType::Unknown,
    }
}

pub fn get_system_drives() -> Vec<DriveInfo> {
    let mut drives = Vec::new();

    unsafe {
        let logical = GetLogicalDrives();

        for i in 0..26u32 {
            if (logical & (1 << i)) == 0 {
                continue;
            }
            let letter = (b'A' + i as u8) as char;
            let drive_path = format!("{}:\\", letter);
            let root_wide: Vec<u16> = drive_path
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let dtype = GetDriveTypeW(PCWSTR(root_wide.as_ptr()));
            if dtype == DRIVE_UNKNOWN || dtype == DRIVE_NO_ROOT_DIR {
                continue;
            }

            let drive_type = map_win_drive_type(dtype);

            let mut total_space: u64 = 0;
            let mut free_space: u64 = 0;
            if GetDiskFreeSpaceExW(
                PCWSTR(root_wide.as_ptr()),
                None,
                Some(&mut total_space),
                Some(&mut free_space),
            )
            .is_err()
            {
                continue;
            }

            let used_space = total_space.saturating_sub(free_space);

            let mut vol_buf = vec![0u16; 256];
            let mut fs_buf = vec![0u16; 64];
            let (volume_label, file_system) = match GetVolumeInformationW(
                PCWSTR(root_wide.as_ptr()),
                Some(&mut vol_buf[..]),
                None,
                None,
                None,
                Some(&mut fs_buf[..]),
            ) {
                Ok(()) => (
                    wide_to_string_lossy(&vol_buf),
                    Some(wide_to_string_lossy(&fs_buf)),
                ),
                Err(_) => (String::new(), None),
            };

            let name = if volume_label.is_empty() {
                format!("Local Disk ({})", letter)
            } else {
                volume_label
            };

            let total_gb = total_space as f64 / (1024.0_f64.powi(3));
            let used_gb = used_space as f64 / (1024.0_f64.powi(3));
            let free_gb = free_space as f64 / (1024.0_f64.powi(3));
            let usage_percent = if total_space > 0 {
                (used_space as f64 / total_space as f64) * 100.0
            } else {
                0.0
            };

            drives.push(DriveInfo {
                path: drive_path.clone(),
                letter: Some(letter),
                name,
                drive_type,
                file_system,
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

    drives.sort_by(|a, b| {
        let rank = |d: &DriveType| match d {
            DriveType::Fixed => 0,
            DriveType::Removable => 1,
            DriveType::Remote => 2,
            DriveType::CDRom => 3,
            _ => 4,
        };
        rank(&a.drive_type)
            .cmp(&rank(&b.drive_type))
            .then(
                a.letter
                    .unwrap_or('\0')
                    .cmp(&b.letter.unwrap_or('\0')),
            )
    });

    drives
}

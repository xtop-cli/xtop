//! Windows mount probe via `GetLogicalDrives` + `GetVolumeInformationW`.
//!
//! Each drive-letter volume maps to its root path (`C:\`) — the mount
//! string sysinfo reports for Windows disks — with an options string built
//! from the genuine volume attributes.

use std::collections::HashMap;

use windows_sys::Win32::Storage::FileSystem::{GetLogicalDrives, GetVolumeInformationW};
use windows_sys::Win32::System::SystemServices::{
    FILE_READ_ONLY_VOLUME, FILE_VOLUME_IS_COMPRESSED,
};

use super::wide;

/// Read-only volumes report `ro`, everything else `rw`; compressed volumes
/// append `compressed`. Other unix-style options have no volume-level
/// equivalent on Windows, so they are not fabricated.
fn format_volume_options(flags: u32) -> String {
    let mut opts = Vec::with_capacity(2);
    if flags & FILE_READ_ONLY_VOLUME != 0 {
        opts.push("ro");
    } else {
        opts.push("rw");
    }
    if flags & FILE_VOLUME_IS_COMPRESSED != 0 {
        opts.push("compressed");
    }
    opts.join(",")
}

fn volume_flags(root: &str) -> Option<u32> {
    let root = wide(root);
    let mut volume_name = vec![0u16; 64];
    let mut file_system = vec![0u16; 32];
    let mut flags = 0u32;
    let ok = unsafe {
        GetVolumeInformationW(
            root.as_ptr(),
            volume_name.as_mut_ptr(),
            volume_name.len() as u32,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut flags,
            file_system.as_mut_ptr(),
            file_system.len() as u32,
        )
    };
    if ok == 0 {
        None
    } else {
        Some(flags)
    }
}

/// Drive-letter volumes keyed by their root path (`C:\`).
pub fn read_mount_options() -> HashMap<String, String> {
    let mut options = HashMap::new();
    let drives = unsafe { GetLogicalDrives() };
    for (i, letter) in (b'A'..=b'Z').enumerate() {
        if drives & (1 << i) == 0 {
            continue;
        }
        let path = format!("{}:\\", letter as char);
        if let Some(flags) = volume_flags(&path) {
            options.insert(path, format_volume_options(flags));
        }
    }
    options
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_options_reflect_read_only_and_compression() {
        assert_eq!(format_volume_options(0), "rw");
        assert_eq!(
            format_volume_options(FILE_VOLUME_IS_COMPRESSED),
            "rw,compressed"
        );
        assert_eq!(
            format_volume_options(FILE_READ_ONLY_VOLUME | FILE_VOLUME_IS_COMPRESSED),
            "ro,compressed"
        );
    }
}

//! Windows thread-count probe.
//!
//! Counts are served from a single system-wide snapshot cached for
//! [`CACHE_TTL`]. Per-process toolhelp snapshots each enumerate the whole
//! system thread table internally (~35 ms per call measured on a
//! ~240-process host; 200 per-pid calls per tick cost ~7 s), while one
//! pass covers every process in milliseconds. The cache also makes the
//! counts of one tick consistent (the per-pid path sampled each process
//! seconds apart).
//!
//! The snapshot prefers the undocumented-but-stable
//! `NtQuerySystemInformation(SystemProcessInformation)` walk (~2.5 ms
//! measured on the same host) over one `pid = 0` toolhelp snapshot
//! (~35 ms). The `UniqueProcessId` field offset differs across Windows
//! builds (0x30 classic, 0x50 on Windows 10 22H2+), so it is calibrated
//! once per process lifetime against the first two NT entries, which are
//! always System Idle (pid 0) and System (pid 4); on 32-bit targets or
//! when the NT call or calibration fails, the toolhelp path is used.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
};

/// How long a cached thread-count snapshot stays fresh.
const CACHE_TTL: Duration = Duration::from_millis(500);

/// Candidate `UniqueProcessId` offsets in `SYSTEM_PROCESS_INFORMATION`.
const PID_OFFSET_CANDIDATES: &[usize] = &[0x30, 0x50];

type CountCache = OnceLock<Mutex<Option<(Instant, HashMap<u32, u64>)>>>;
static CACHE: CountCache = OnceLock::new();

/// Calibrated once: the `UniqueProcessId` offset, or `None` when the NT
/// path is unusable on this build (toolhelp fallback).
static PID_OFFSET: OnceLock<Option<usize>> = OnceLock::new();

/// Thread count of one process; 0 for inaccessible or vanished processes
/// (system processes like PID 0/4 fall here, mirroring unix behavior).
pub fn read_thread_count(pid: sysinfo::Pid) -> u64 {
    let cache = CACHE.get_or_init(|| Mutex::new(None));
    let mut guard = cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let now = Instant::now();
    let stale = match guard.as_ref() {
        Some((at, _)) => now.duration_since(*at) >= CACHE_TTL,
        None => true,
    };
    if stale {
        *guard = Some((now, snapshot_all_thread_counts()));
    }
    guard
        .as_ref()
        .and_then(|(_, counts)| counts.get(&pid.as_u32()))
        .copied()
        .unwrap_or(0)
}

/// One system-wide process/thread snapshot as `process pid → thread count`.
fn snapshot_all_thread_counts() -> HashMap<u32, u64> {
    nt_snapshot_counts().unwrap_or_else(toolhelp_snapshot_counts)
}

/// `NtQuerySystemInformation(SystemProcessInformation)` walk (64-bit only;
/// 32-bit targets keep the toolhelp path).
#[cfg(target_pointer_width = "64")]
fn nt_snapshot_counts() -> Option<HashMap<u32, u64>> {
    const STATUS_INFO_LENGTH_MISMATCH: i32 = 0xC000_0004u32 as i32;
    const ENTRY_NEXT: usize = 0;
    const ENTRY_THREADS: usize = 4;

    unsafe extern "system" {
        fn NtQuerySystemInformation(
            class: u32,
            info: *mut std::ffi::c_void,
            len: u32,
            return_len: *mut u32,
        ) -> i32;
    }

    let mut needed = 0u32;
    let mut buf: Vec<u8> = Vec::new();
    loop {
        let status = unsafe {
            NtQuerySystemInformation(5, buf.as_mut_ptr().cast(), buf.len() as u32, &mut needed)
        };
        if status == 0 {
            break;
        }
        if status == STATUS_INFO_LENGTH_MISMATCH && needed as usize > buf.len() {
            buf.resize(needed as usize, 0);
            continue;
        }
        return None;
    }

    let pid_off = (*PID_OFFSET.get_or_init(|| calibrate_pid_offset(&buf)))?;
    let entry_len = pid_off.checked_add(std::mem::size_of::<u64>())?;
    let mut counts: HashMap<u32, u64> = HashMap::new();
    let base = buf.as_ptr();
    let mut off = 0usize;
    loop {
        // The buffer comes from the kernel, but never read past it if the
        // reported `NextEntryOffset` chain is truncated or inconsistent.
        if off.checked_add(entry_len)? > buf.len() {
            return None;
        }
        let entry = unsafe { base.add(off) };
        let next =
            unsafe { std::ptr::read_unaligned(entry.add(ENTRY_NEXT) as *const u32) } as usize;
        let threads =
            unsafe { std::ptr::read_unaligned(entry.add(ENTRY_THREADS) as *const u32) } as u64;
        let pid = unsafe { std::ptr::read_unaligned(entry.add(pid_off) as *const u64) } as u32;
        if threads > 0 {
            counts.insert(pid, threads);
        }
        if next == 0 {
            break;
        }
        off = off.checked_add(next)?;
    }
    Some(counts)
}

/// 32-bit targets keep the toolhelp path (the NT layout offsets differ).
#[cfg(target_pointer_width = "32")]
fn nt_snapshot_counts() -> Option<HashMap<u32, u64>> {
    None
}

/// Pick the `UniqueProcessId` offset whose first two entries read as the
/// always-present System Idle (pid 0) and System (pid 4) processes;
/// `None` when no candidate matches (toolhelp fallback).
#[cfg(target_pointer_width = "64")]
fn calibrate_pid_offset(buf: &[u8]) -> Option<usize> {
    // Need room for at least the first entry's pid field.
    if buf.len() < PID_OFFSET_CANDIDATES[1] + 8 {
        return None;
    }
    for &candidate in PID_OFFSET_CANDIDATES {
        let Some(entry_len) = candidate.checked_add(std::mem::size_of::<u64>()) else {
            continue;
        };
        let mut off = 0usize;
        let mut first_two = [0u32; 2];
        let mut seen = 0usize;
        let mut valid = true;
        loop {
            let Some(end) = off.checked_add(entry_len) else {
                valid = false;
                break;
            };
            if end > buf.len() {
                valid = false;
                break;
            }
            let entry = unsafe { buf.as_ptr().add(off) };
            let next = unsafe { std::ptr::read_unaligned(entry as *const u32) } as usize;
            let pid =
                unsafe { std::ptr::read_unaligned(entry.add(candidate) as *const u64) } as u32;
            if seen < 2 {
                first_two[seen] = pid;
            }
            seen += 1;
            if next == 0 {
                break;
            }
            let Some(next_off) = off.checked_add(next) else {
                valid = false;
                break;
            };
            off = next_off;
        }
        if valid && first_two == [0, 4] && seen >= 2 {
            return Some(candidate);
        }
    }
    None
}

#[cfg(target_pointer_width = "32")]
fn calibrate_pid_offset(_buf: &[u8]) -> Option<usize> {
    None
}

/// Fallback: one system-wide toolhelp thread snapshot.
fn toolhelp_snapshot_counts() -> HashMap<u32, u64> {
    let mut counts: HashMap<u32, u64> = HashMap::new();
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        if snapshot == INVALID_HANDLE_VALUE || snapshot.is_null() {
            return counts;
        }
        let mut entry = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            ..std::mem::zeroed()
        };
        if Thread32First(snapshot, &mut entry) != 0 {
            loop {
                *counts.entry(entry.th32OwnerProcessID).or_insert(0) += 1;
                if Thread32Next(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pid_offset_calibration_accepts_known_first_entries() {
        // Synthetic SYSTEM_PROCESS_INFORMATION buffer: two entries with
        // pid 0 then 4 at offset 0x50 (Windows 10 22H2+ layout).
        let mut buf = vec![0u8; 200];
        let put = |buf: &mut [u8], off: usize, next: u32, pid: u64| {
            buf[off..off + 4].copy_from_slice(&next.to_le_bytes());
            buf[off + 4..off + 8].copy_from_slice(&3u32.to_le_bytes());
            buf[off + 0x50..off + 0x58].copy_from_slice(&pid.to_le_bytes());
        };
        put(&mut buf, 0, 96, 0);
        put(&mut buf, 96, 0, 4);
        assert_eq!(calibrate_pid_offset(&buf), Some(0x50));

        // Rebuild with the pid at 0x30 instead: the classic layout wins.
        let mut classic = vec![0u8; 200];
        let put32 = |buf: &mut [u8], off: usize, next: u32, pid: u64| {
            buf[off..off + 4].copy_from_slice(&next.to_le_bytes());
            buf[off + 4..off + 8].copy_from_slice(&3u32.to_le_bytes());
            buf[off + 0x30..off + 0x38].copy_from_slice(&pid.to_le_bytes());
        };
        put32(&mut classic, 0, 96, 0);
        put32(&mut classic, 96, 0, 4);
        assert_eq!(calibrate_pid_offset(&classic), Some(0x30));
    }

    #[test]
    fn pid_offset_calibration_rejects_unknown_layouts() {
        let mut buf = vec![0u8; 96];
        buf[4..8].copy_from_slice(&3u32.to_le_bytes());
        buf[0x50..0x58].copy_from_slice(&999u64.to_le_bytes());
        assert_eq!(calibrate_pid_offset(&buf), None);
        assert_eq!(calibrate_pid_offset(&[]), None);
    }
}

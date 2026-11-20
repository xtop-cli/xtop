//! OS-specific probes for the [`super`] sysinfo area.
//!
//! Every platform module below provides the same function set. The modules
//! that are not compiled for the current target are simply not exported;
//! `cfg` dispatch happens here in one place.
//!
//! [`shared`] holds probe logic used by more than one platform.
//!
//! Platform notes:
//! - **Linux**: reads `/sys` and `/proc` directly (governor, batteries,
//!   interface addresses, threads, DRM GPUs).
//! - **macOS**: real probes per subsystem under `macos/` — batteries via
//!   `pmset`, interface IPs via `getifaddrs`, mount options via `mount(8)`,
//!   thread counts via `proc_pidinfo`, Directory Services users via `dscl`.
//!   Per-core temps, package power and GPU info stay empty by design (see
//!   `macos/mod.rs`).
//! - **Windows**: real probes under `windows/` — batteries via
//!   `GetSystemPowerStatus`, interface IPs via `GetAdaptersAddresses`,
//!   mount options via `GetLogicalDrives`/`GetVolumeInformationW`, thread
//!   counts via toolhelp snapshots, local users via `Get-LocalUser` and
//!   process user ids exposed as numeric RIDs (sysinfo reports SIDs).
//!   Per-core temps, package power and non-NVIDIA GPU info stay empty by
//!   design (see `windows/mod.rs`).

pub mod shared;

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod fallback;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub use fallback::*;
#[cfg(target_os = "linux")]
pub use linux::*;
#[cfg(target_os = "macos")]
pub use macos::*;
#[cfg(target_os = "windows")]
pub use windows::*;

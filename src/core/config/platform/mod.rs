//! Config directory resolution per platform.
//!
//! Only here lives OS-specific path logic. Shared helpers used by every
//! platform live under [`shared`].

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod other;
pub mod shared;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::config_dir;
#[cfg(target_os = "macos")]
pub use macos::config_dir;
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub use other::config_dir;
#[cfg(target_os = "windows")]
pub use windows::config_dir;

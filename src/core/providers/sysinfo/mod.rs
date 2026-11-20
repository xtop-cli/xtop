//! System metrics provider for the `xtop` kernel.
//!
//! The real-time monitoring data comes from the `sysinfo` crate
//! (cross-platform) plus a small set of OS-specific probes defined under
//! [`platform`]. Platform modules must compile for every supported target and
//! fall back to empty/default values when the OS does not provide the data.

pub mod platform;

mod provider;

pub use provider::SysinfoProvider;

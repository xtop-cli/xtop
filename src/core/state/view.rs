//! View-control state shared between the kernel state and the UI.
//!
//! Fullscreen targets, input modes and the command palette state.

use crate::core::config::keybinding::Action;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FullScreenWidget {
    #[default]
    None,
    Cpu,
    Memory,
    Storage,
    Network,
    Processes,
    DiskIO,
    Gpu,
    Battery,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProcessSortBy {
    Cpu,
    Memory,
    Pid,
    Name,
}

impl ProcessSortBy {
    pub fn next(self) -> Self {
        match self {
            Self::Cpu => Self::Memory,
            Self::Memory => Self::Pid,
            Self::Pid => Self::Name,
            Self::Name => Self::Cpu,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Cpu => "CPU%",
            Self::Memory => "Mem",
            Self::Pid => "PID",
            Self::Name => "Name",
        }
    }
}

impl FullScreenWidget {
    pub fn next(self) -> Self {
        match self {
            Self::None => Self::Cpu,
            Self::Cpu => Self::Memory,
            Self::Memory => Self::Storage,
            Self::Storage => Self::Network,
            Self::Network => Self::Processes,
            Self::Processes => Self::DiskIO,
            Self::DiskIO => Self::Gpu,
            Self::Gpu => Self::Battery,
            Self::Battery => Self::None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Cpu => "CPU",
            Self::Memory => "Memory",
            Self::Storage => "Storage",
            Self::Network => "Network",
            Self::Processes => "Processes",
            Self::DiskIO => "Disk I/O",
            Self::Gpu => "GPU",
            Self::Battery => "Battery",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaletteEntry {
    pub label: String,
    pub action: Action,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PalettePage {
    Main,
    Themes,
    Layouts,
    Plugins,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaletteState {
    pub open: bool,
    pub query: String,
    pub selected: usize,
    pub entries: Vec<PaletteEntry>,
    pub filtered: Vec<usize>,
    pub page: PalettePage,
}

impl PaletteState {
    pub fn title(&self) -> &str {
        match self.page {
            PalettePage::Main => "Command Bar",
            PalettePage::Themes => "Select Theme",
            PalettePage::Layouts => "Select Layout",
            PalettePage::Plugins => "Plugins",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Searching,
    CommandPalette,
}

//! Command palette state logic (open/navigate/filter/execute pages).
//!
//! Lives in its own module to keep `AppState` focused; these functions only
//! touch public state fields, so they are plain `impl` additions.

use crate::core::config::keybinding::Action;
use crate::core::state::view::{InputMode, PaletteEntry, PalettePage};

use crate::core::state::AppState;

impl AppState {
    pub(crate) fn rebuild_palette(&mut self) {
        self.palette.entries.clear();
        match self.palette.page {
            PalettePage::Main => {
                self.palette.entries.push(PaletteEntry {
                    label: "Themes →".into(),
                    action: Action::NavigateThemes,
                });
                self.palette.entries.push(PaletteEntry {
                    label: "Layouts →".into(),
                    action: Action::NavigateLayouts,
                });
                self.palette.entries.push(PaletteEntry {
                    label: "Toggle Fullscreen".into(),
                    action: Action::ToggleFullscreen,
                });
                self.palette.entries.push(PaletteEntry {
                    label: "Cycle Fullscreen Widget".into(),
                    action: Action::CycleFullscreen,
                });
                self.palette.entries.push(PaletteEntry {
                    label: "Plugins →".into(),
                    action: Action::NavigatePlugins,
                });
                self.palette.entries.push(PaletteEntry {
                    label: "Processes (filter) →".into(),
                    action: Action::Search,
                });
                self.palette.entries.push(PaletteEntry {
                    label: "Toggle Help".into(),
                    action: Action::ToggleHelp,
                });
                self.palette.entries.push(PaletteEntry {
                    label: format!(
                        "Sort: {} {}",
                        self.process_sort.label(),
                        if self.process_sort_desc { "▼" } else { "▲" }
                    ),
                    action: Action::SortByCpu,
                });
                self.palette.entries.push(PaletteEntry {
                    label: "Random Theme".into(),
                    action: Action::RandomTheme,
                });
                self.palette.entries.push(PaletteEntry {
                    label: "Exit".into(),
                    action: Action::Quit,
                });
            }
            PalettePage::Themes => {
                for (i, theme) in self.themes.iter().enumerate() {
                    self.palette.entries.push(PaletteEntry {
                        label: theme.name.clone(),
                        action: Action::SelectTheme(i),
                    });
                }
            }
            PalettePage::Layouts => {
                for (i, layout) in self.layout_defs.iter().enumerate() {
                    self.palette.entries.push(PaletteEntry {
                        label: layout.name.clone(),
                        action: Action::SelectLayout(i),
                    });
                }
            }
            PalettePage::Plugins => {
                let plugins = self.plugin_infos();
                if plugins.is_empty() {
                    self.palette.entries.push(PaletteEntry {
                        label: "(no plugins registered)".into(),
                        action: Action::Noop,
                    });
                } else {
                    for plugin in plugins {
                        self.palette.entries.push(PaletteEntry {
                            label: format!(
                                "{} {} ({})",
                                if plugin.enabled { "Disable" } else { "Enable" },
                                plugin.name,
                                plugin.id
                            ),
                            action: Action::TogglePlugin(plugin.id),
                        });
                    }
                }
            }
        }
        self.palette_filter();
    }

    pub(crate) fn open_palette(&mut self) {
        self.palette.open = true;
        self.palette.query.clear();
        self.palette.selected = 0;
        self.palette.page = PalettePage::Main;
        self.rebuild_palette();
    }

    pub(crate) fn palette_navigate_to(&mut self, page: PalettePage) {
        self.palette.page = page;
        self.palette.query.clear();
        self.palette.selected = 0;
        self.rebuild_palette();
    }

    pub(crate) fn palette_filter(&mut self) {
        let q = self.palette.query.to_lowercase();
        self.palette.filtered = self
            .palette
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| q.is_empty() || e.label.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect();
        if !self.palette.filtered.is_empty() {
            self.palette.selected = self.palette.selected.min(self.palette.filtered.len() - 1);
        } else {
            self.palette.selected = 0;
        }
    }

    pub(crate) fn palette_select_next(&mut self) {
        if !self.palette.filtered.is_empty() {
            self.palette.selected = (self.palette.selected + 1) % self.palette.filtered.len();
        }
    }

    pub(crate) fn palette_select_prev(&mut self) {
        if !self.palette.filtered.is_empty() {
            self.palette.selected = if self.palette.selected == 0 {
                self.palette.filtered.len() - 1
            } else {
                self.palette.selected - 1
            };
        }
    }

    pub(crate) fn palette_selected_action(&self) -> Option<Action> {
        self.palette
            .filtered
            .get(self.palette.selected)
            .and_then(|&i| self.palette.entries.get(i))
            .map(|e| e.action.clone())
    }

    pub(crate) fn close_palette(&mut self) {
        self.palette.open = false;
        self.palette.page = PalettePage::Main;
        self.input_mode = InputMode::Normal;
    }
}

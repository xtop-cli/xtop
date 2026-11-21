//! TUI action dispatch: translate config keybinding actions into state
//! mutations. These are frontend concerns; the CLI does not use them.

use crate::core::config::keybinding::Action;
use crate::core::state::view::{FullScreenWidget, InputMode, PalettePage};
use crate::core::state::AppState;

impl AppState {
    pub(crate) fn execute_action(&mut self, action: &Action) {
        match action {
            Action::Quit => self.quit(),
            Action::ToggleHelp => self.toggle_help(),
            Action::NextTheme => self.next_theme(),
            Action::PreviousTheme => self.previous_theme(),
            Action::NextLayout => self.next_layout(),
            Action::ToggleFullscreen => self.toggle_fullscreen(),
            Action::CycleFullscreen => self.cycle_fullscreen_widget(),
            Action::Search => {
                self.close_palette();
                self.start_search();
                return;
            }
            Action::OpenCommandPalette => {}
            Action::Cancel => {
                if self.show_help {
                    self.toggle_help();
                }
            }
            Action::SelectTheme(i) => {
                self.selected_theme_index = *i;
                self.apply_theme();
            }
            Action::SelectLayout(i) => {
                if *i < self.layout_defs.len() {
                    self.layout_index = *i;
                    self.sync_layout_mode();
                    self.full_screen_widget = FullScreenWidget::None;
                }
            }
            Action::NavigateThemes => {
                self.palette_navigate_to(PalettePage::Themes);
                return;
            }
            Action::NavigateLayouts => {
                self.palette_navigate_to(PalettePage::Layouts);
                return;
            }
            Action::NavigatePlugins => {
                self.palette_navigate_to(PalettePage::Plugins);
                return;
            }
            Action::TogglePlugin(id) => {
                let enabled = !self.is_plugin_enabled(id);
                let _ = self.set_plugin_enabled(id, enabled);
                self.rebuild_palette();
                return;
            }
            Action::Noop => return,
            Action::KillProcess => {
                if let Some(pid) = self.process_selected_pid {
                    self.kill_process_by_pid(pid);
                    self.process_selected_pid = None;
                }
            }
            Action::ProcessUp => self.process_select_prev(),
            Action::ProcessDown => self.process_select_next(),
            Action::SortByCpu => {
                self.cycle_sort();
            }
            Action::RandomTheme => {
                let n = self.themes.len();
                if n > 1 {
                    let next = (self.selected_theme_index + 7) % n;
                    self.selected_theme_index = next;
                    self.apply_theme();
                }
            }
        }
        // Close palette after executing any action (except navigation which returns above)
        if self.input_mode == InputMode::CommandPalette {
            self.close_palette();
        }
    }
}

//! TUI key mapping and mode dispatch.

use crate::core::config::keybinding::Action;
use crate::core::state::save_config;
use crate::core::state::{AppState, InputMode, PalettePage};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(crate) fn key_event_to_str(key: &KeyEvent) -> String {
    let mut s = String::new();
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    if ctrl {
        s.push_str("ctrl+");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        s.push_str("alt+");
    }
    match key.code {
        KeyCode::Char(c) => {
            if ctrl {
                s.push(c.to_ascii_lowercase());
            } else {
                s.push(c);
            }
        }
        KeyCode::Esc => s.push_str("escape"),
        KeyCode::Enter => s.push_str("enter"),
        KeyCode::Backspace => s.push_str("backspace"),
        KeyCode::Tab => s.push_str("tab"),
        KeyCode::Up => s.push_str("up"),
        KeyCode::Down => s.push_str("down"),
        KeyCode::Left => s.push_str("left"),
        KeyCode::Right => s.push_str("right"),
        KeyCode::Delete => s.push_str("delete"),
        KeyCode::Home => s.push_str("home"),
        KeyCode::End => s.push_str("end"),
        KeyCode::PageUp => s.push_str("pageup"),
        KeyCode::PageDown => s.push_str("pagedown"),
        _ => return String::new(),
    }
    s
}

pub(crate) fn handle_key(state: &mut AppState, key: KeyEvent) {
    let key_str = key_event_to_str(&key);
    let ctrl_c = key_str == "ctrl+c";

    match state.input_mode {
        InputMode::Normal => {
            if ctrl_c {
                save_config(state);
                state.quit();
                return;
            }
            // Direct Ctrl+P check (works regardless of keybinding config, important on macOS)
            if key_str == "ctrl+p" {
                state.open_palette();
                state.input_mode = InputMode::CommandPalette;
                return;
            }
            // Give plugins a chance to consume keys only in Normal mode, so a
            // plugin key handler can never eat typing inside search/palette.
            let key_consumed = state
                .with_plugin_manager_mut(|mgr, this| mgr.handle_key(this, &key_str))
                .unwrap_or(false);
            if key_consumed {
                return;
            }

            if cfg!(debug_assertions) && !key_str.is_empty() {
                eprintln!("[key] '{key_str}'");
            }

            if let Some(action) = state.keybindings.resolve(&key_str) {
                match action {
                    Action::Quit => {
                        save_config(state);
                        state.quit();
                    }
                    Action::Cancel if state.show_help => {
                        state.toggle_help();
                    }
                    Action::OpenCommandPalette => {
                        state.open_palette();
                        state.input_mode = InputMode::CommandPalette;
                    }
                    _ => {
                        state.execute_action(&action);
                        if action.persists() {
                            save_config(state);
                        }
                    }
                }
            }
        }
        InputMode::Searching => {
            if ctrl_c || key.code == KeyCode::Esc {
                state.search_query.clear();
                state.end_search();
                return;
            }
            match key.code {
                KeyCode::Enter => {
                    state.end_search();
                }
                KeyCode::Backspace => {
                    state.search_pop_char();
                }
                KeyCode::Char(c) => {
                    state.search_push_char(c);
                }
                _ => {}
            }
        }
        InputMode::CommandPalette => {
            let is_main = state.palette.page == PalettePage::Main;
            if ctrl_c || key.code == KeyCode::Esc {
                state.close_palette();
                return;
            }
            match key.code {
                KeyCode::Enter => {
                    if let Some(action) = state.palette_selected_action() {
                        state.execute_action(&action);
                        if action.persists() {
                            save_config(state);
                        }
                    }
                }
                KeyCode::Down => {
                    state.palette_select_next();
                }
                KeyCode::Up => {
                    state.palette_select_prev();
                }
                KeyCode::Char(c) => {
                    state.palette.query.push(c);
                    state.palette_filter();
                }
                KeyCode::Backspace => {
                    if state.palette.query.is_empty() && !is_main {
                        state.palette_navigate_to(PalettePage::Main);
                    } else {
                        state.palette.query.pop();
                        state.palette_filter();
                    }
                }
                _ => {}
            }
        }
    }
}

//! The interactive TUI run loop.

use std::time::{Duration, Instant};

use crate::config::config_dir;
use crate::config::keybinding::Action;
use crate::state::{InputMode, PalettePage};
use crate::ui;
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind,
};

use super::share::{initialize_state, save_config};
use super::theme::persisted_theme;

fn key_event_to_str(key: &KeyEvent) -> String {
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

/// Run the interactive TUI loop.
pub fn run() -> anyhow::Result<()> {
    ui::install_panic_hook();
    let mut terminal = ui::init()?;

    let cfg_dir = config_dir();
    let mut state = initialize_state(&cfg_dir)?;

    // Frame effects read the persisted `effect` key once at startup (same
    // config file `initialize_state` already loaded). The host is
    // feature-agnostic: without `--features effects` it is an idle no-op.
    let mut effect_host =
        ui::effects::EffectHost::from_config(crate::config::load_config().effect.as_deref());

    // Sample once before the first frame so the UI never shows an empty
    // snapshot and every widget shares the same per-tick data.
    state.on_tick();
    let mut last_tick = Instant::now();

    loop {
        let tick_rate = Duration::from_millis(state.update_interval_ms.max(100));
        terminal.draw(|f| {
            ui::render(f, &mut state);
            // Effects transform the fully rendered frame (after layout,
            // before flush); see the `xtop-effect-api` host contract.
            effect_host.apply(f.buffer_mut());
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_default();

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    handle_key(&mut state, key);
                }
                Event::Paste(text) => {
                    // Bracketed paste: insert as text in the current input.
                    match state.input_mode {
                        InputMode::Searching => {
                            for c in text.chars() {
                                state.search_push_char(c);
                            }
                        }
                        InputMode::CommandPalette => {
                            state.palette.query.push_str(&text);
                            state.palette_filter();
                        }
                        InputMode::Normal => {}
                    }
                }
                Event::Mouse(mouse)
                    if state.input_mode == InputMode::Normal && !state.show_help =>
                {
                    // Mouse capture is enabled mainly for wheel scrolling of
                    // the process list in Normal mode.
                    match mouse.kind {
                        MouseEventKind::ScrollUp => state.process_select_prev(),
                        MouseEventKind::ScrollDown => state.process_select_next(),
                        _ => {}
                    }
                }
                // Resize triggers a redraw on the next loop iteration
                // (ratatui re-measures inside `draw`).
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            state.on_tick();
            sync_external_theme(&mut state);
            last_tick = Instant::now();
        }

        if state.should_quit {
            break;
        }
    }

    // Disable plugins on shutdown
    let _ = state.with_plugin_manager_mut(|mgr, this| {
        mgr.disable_all(this);
    });

    ui::restore()?;
    Ok(())
}

/// Apply a theme changed by another process (`xtop --ct <name>`).
///
/// The CLI persists the choice in `config.json`; polling the file at tick
/// boundaries gives live sync from another terminal without IPC. A missing or
/// invalid config file is ignored so a partial write never resets the theme.
fn sync_external_theme(state: &mut crate::state::AppState) {
    if let Some(name) = persisted_theme() {
        if name != state.current_theme.name {
            state.set_theme_by_name(&name);
        }
    }
}

fn handle_key(state: &mut crate::state::AppState, key: KeyEvent) {
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

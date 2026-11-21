//! The interactive TUI run loop.

use std::time::{Duration, Instant};

use crate::core::bootstrap::{initialize_state, load_layout_defs};
use crate::core::config::{config_dir, load_config, persisted_selection};
use crate::core::state::{AppState, InputMode};
use crate::core::theme::load_all_themes;
use crossterm::event::{self, Event, KeyEventKind, MouseEventKind};

use super::effects::EffectHost;
use super::input::handle_key;
use super::screen;
use super::terminal;

/// Run the interactive TUI loop, restoring the terminal on every outcome.
pub fn run() -> anyhow::Result<()> {
    terminal::install_panic_hook();
    let mut terminal = terminal::init()?;
    let result = run_session(&mut terminal);
    let restored = terminal::restore();
    match (result, restored) {
        (Err(error), _) => Err(error),
        (Ok(()), Err(error)) => Err(error.into()),
        (Ok(()), Ok(())) => Ok(()),
    }
}

fn run_session(terminal: &mut terminal::Tui) -> anyhow::Result<()> {
    let cfg_dir = config_dir();
    let mut state = initialize_state(&cfg_dir)?;

    // Frame effects read the persisted `effect` key once at startup (same
    // config file `initialize_state` already loaded). The host is
    // feature-agnostic: without `--features effects` it is an idle no-op.
    let mut effect_host = EffectHost::from_config(load_config().effect.as_deref());

    // Sample once before the first frame so the UI never shows an empty
    // snapshot and every widget shares the same per-tick data.
    state.on_tick();
    let mut last_tick = Instant::now();

    loop {
        let tick_rate = Duration::from_millis(state.update_interval_ms.max(100));
        terminal.draw(|f| {
            screen::render(f, &mut state);
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
            sync_external_state(&mut state);
            last_tick = Instant::now();
        }

        if state.should_quit {
            break;
        }
    }

    // Disable plugins on shutdown.
    let _ = state.with_plugin_manager_mut(|mgr, this| {
        mgr.disable_all(this);
    });

    Ok(())
}

/// Apply theme and layout changes persisted by another process.
///
/// The CLI writes `config.json`; polling it at tick boundaries gives live sync
/// without IPC. Newly downloaded assets are reloaded when the persisted name
/// is not in the current state, so `theme download`/`layout download` followed
/// by `set` works against a running TUI.
fn sync_external_state(state: &mut AppState) {
    let Some(selection) = persisted_selection() else {
        return;
    };
    if selection.theme != state.current_theme.name && !state.set_theme_by_name(&selection.theme) {
        state.themes = load_all_themes();
        let _ = state.set_theme_by_name(&selection.theme);
    }
    if !selection.layout.is_empty()
        && selection.layout != state.current_layout_name()
        && !state.set_layout_by_name(&selection.layout)
    {
        state.layout_defs = load_layout_defs();
        let _ = state.set_layout_by_name(&selection.layout);
    }
}

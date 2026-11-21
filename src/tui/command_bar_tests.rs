//! Command bar and plugin-toggle interaction tests.

use crate::core::config::keybinding::Action;
use crate::core::config::Config;
use crate::core::providers::sysinfo::SysinfoProvider;
use crate::core::state::{AppState, InputMode, PalettePage};
use crate::core::theme::Theme;
use crate::tui::input::handle_key;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use xtop_layout::default_layouts;

fn state() -> AppState {
    let theme = Theme {
        name: "test".into(),
        background: [0, 0, 0],
        foreground: [255, 255, 255],
        palette: [[0, 0, 0]; 16],
    };
    AppState::new(
        Box::new(SysinfoProvider::new()),
        vec![theme],
        Config::default(),
        default_layouts(),
    )
}

#[test]
fn command_bar_exposes_groups_and_process_filter() {
    let mut state = state();
    state.open_palette();
    let labels: Vec<&str> = state
        .palette
        .entries
        .iter()
        .map(|entry| entry.label.as_str())
        .collect();
    assert!(labels.contains(&"Themes →"));
    assert!(labels.contains(&"Layouts →"));
    assert!(labels.contains(&"Plugins →"));
    assert!(labels.contains(&"Processes (filter) →"));

    state.palette_navigate_to(PalettePage::Plugins);
    assert!(state
        .palette
        .entries
        .iter()
        .any(|entry| entry.label.contains("no plugins registered")));

    state.open_palette();
    state.execute_action(&Action::Search);
    assert!(!state.palette.open);
    assert_eq!(state.input_mode, InputMode::Searching);
}

#[test]
fn slash_opens_command_bar_and_ctrl_f_filters_processes() {
    let mut state = state();
    handle_key(
        &mut state,
        KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
    );
    assert!(state.palette.open);
    assert_eq!(state.input_mode, InputMode::CommandPalette);

    handle_key(&mut state, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(!state.palette.open);
    assert_eq!(state.input_mode, InputMode::Normal);

    handle_key(
        &mut state,
        KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL),
    );
    assert_eq!(state.input_mode, InputMode::Searching);
}

#[test]
fn command_bar_clears_its_centered_frame() {
    use crate::tui::overlay::palette;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::widgets::Paragraph;
    use ratatui::Terminal;

    let mut state = state();
    handle_key(
        &mut state,
        KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
    );
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let popup = palette::popup_area(Rect::new(0, 0, 80, 24));
    terminal
        .draw(|f| {
            f.render_widget(Paragraph::new("UNDERLAY"), popup);
            crate::tui::screen::render(f, &mut state);
        })
        .unwrap();

    let rendered: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect();
    let frame: String = (popup.y..popup.bottom())
        .flat_map(|y| (popup.x..popup.right()).map(move |x| (x, y)))
        .map(|(x, y)| terminal.backend().buffer()[(x, y)].symbol())
        .collect();
    assert!(!frame.contains("UNDERLAY"));
    assert!(rendered.contains("Command Bar"));
    assert!(popup.width < 80 && popup.height < 24);
}

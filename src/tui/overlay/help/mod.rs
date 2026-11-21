//! Help widget: keybinding reference overlay.
//!
//! Built from the *live* `Keybindings` (config-driven), so remapped keys are
//! always reflected here.
//!
//! Chrome (DR-UX3): title and key spans use the accent role, separators and
//! secondary notes the dim role, and the block border follows the user's
//! global `style.borders` choice — same look as widget frames.

use crate::core::state::AppState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use xtop_widget_api::glyph::{border_for, to_color};

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let fg = to_color(*state.current_theme.fg());
    let bg = to_color(*state.current_theme.bg());
    let accent = to_color(*state.current_theme.accent());
    let dim = to_color(*state.current_theme.dim());
    let kb = &state.keybindings;

    let mut text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Keybindings",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  ─────────────────────────────────────────────",
            Style::default().fg(dim),
        )]),
    ];
    push_key(&mut text, "Quit", &kb.quit, accent);
    push_key(&mut text, "Help", &kb.help, accent);
    text.push(Line::from(""));
    push_key(&mut text, "Next theme", &kb.next_theme, accent);
    push_key(&mut text, "Previous theme", &kb.prev_theme, accent);
    push_key(&mut text, "Next layout", &kb.next_layout, accent);
    push_key(
        &mut text,
        "Toggle fullscreen",
        &kb.toggle_fullscreen,
        accent,
    );
    push_key(&mut text, "Cycle fullscreen", &kb.cycle_fullscreen, accent);
    text.push(Line::from(""));
    push_key(&mut text, "Filter processes", &kb.search, accent);
    push_key(&mut text, "Command bar", &kb.command_palette, accent);
    push_key(&mut text, "Cancel", &kb.cancel, accent);
    text.push(Line::from(""));
    push_key(&mut text, "Kill process", &kb.kill_process, accent);
    push_key(&mut text, "Select up", &kb.process_up, accent);
    push_key(&mut text, "Select down", &kb.process_down, accent);
    push_key(&mut text, "Cycle sort", &kb.cycle_sort, accent);
    text.push(Line::from(vec![Span::styled(
        "    first press flips the current column (▼ -> ▲); the next press\n\
         \x20   moves to the next column, descending first (CPU% -> Mem -> PID -> Name)",
        Style::default().fg(dim),
    )]));
    text.extend([
        Line::from(""),
        Line::from(vec![Span::styled(
            "  ─────────────────────────────────────────────",
            Style::default().fg(dim),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Layouts",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        )]),
        Line::from(format!("    Current: {}", state.current_layout_name())),
        Line::from("    Modes: Dashboard | Vertical | Horizontal | CPU Focus"),
        Line::from("    Memory Focus | Network Focus | Process Focus"),
        Line::from("    Presets: Detail Dashboard | Detail Network | Detail Processes"),
        Line::from(format!(
            "    + custom layouts from {}/",
            crate::core::config::config_dir().join("layouts").display()
        )),
        Line::from(""),
        Line::from("  https://github.com/xtop-cli/xtop"),
    ]);

    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            " Help ",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        )]))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_set(border_for(state.style.borders))
        .border_style(Style::default().fg(accent))
        .style(Style::default().fg(fg).bg(bg));
    let p = Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(fg).bg(bg))
        .wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

fn push_key(text: &mut Vec<Line<'static>>, action: &str, keys: &[String], accent: Color) {
    let rendered = if keys.is_empty() {
        "(unbound)".to_string()
    } else {
        keys.join(", ")
    };
    text.push(Line::from(vec![
        Span::styled(format!("  {rendered:<12}"), Style::default().fg(accent)),
        Span::raw(action.to_string()),
    ]));
}

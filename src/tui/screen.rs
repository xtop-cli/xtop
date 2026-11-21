//! Screen renderer: dispatches to layouts, fullscreen, overlays and the
//! minimal view. Data widgets are drawn through the widget packs resolved by
//! the engine; overlays (help/palette) are kernel-owned.

use crate::core::state::{AppState, FullScreenWidget, InputMode};
use crate::tui::layout::{render_layout, render_named, widget_node_options};
use crate::tui::overlay::{help, palette};
use ratatui::prelude::*;
use ratatui::Frame;
use std::collections::HashMap;
use xtop_layout::{detect_effective_layout, EffectiveLayout};
use xtop_plugin_api::PluginWidget;
use xtop_widget_api::glyph::{border_for, to_color};

/// Build a plugin widget lookup map from AppState.
///
/// Plugin renderers only see [`HostState`](xtop_plugin_api::HostState);
/// they keep precedence over every pack.
fn plugin_widgets(state: &AppState) -> HashMap<String, PluginWidget> {
    let mut map: HashMap<String, PluginWidget> = HashMap::new();
    for reg in &state.plugin_widgets {
        // `PluginWidget` is not `Clone` by contract; the render closure
        // (an `Arc`) is, so rebuild a lightweight copy per frame.
        map.insert(
            reg.name.clone(),
            PluginWidget {
                name: reg.name.clone(),
                render: reg.render.clone(),
            },
        );
    }
    map
}

/// Options of the first widget node named `name` in the current layout
/// (DR-UX1). Fullscreen/minimal widgets are rendered by name, so their
/// display options come from the layout they would render under; `None`
/// when the current layout has no such node (default behavior).
fn current_widget_options(state: &AppState, name: &str) -> Option<serde_json::Value> {
    widget_node_options(&state.current_layout().root, name).cloned()
}

pub fn render(f: &mut Frame, state: &mut AppState) {
    let area = f.area();

    if area.width < 40 || area.height < 8 {
        render_too_small(f, state, area);
        return;
    }

    if state.show_help {
        help::render(f, state, area);
        return;
    }

    if state.full_screen_widget != FullScreenWidget::None {
        render_fullscreen(f, state, area);
        return;
    }

    let mode = detect_effective_layout(area.width, area.height, state.layout_mode);
    let pw = plugin_widgets(state);

    if mode == EffectiveLayout::Minimal {
        render_minimal(f, state, area);
    } else {
        // The engine walks the layout tree while widget renderers run
        // against `state`; hand it an unaliased copy of the current layout
        // definition (small: a bounded tree of node names).
        let def = state.current_layout().clone();
        render_layout(f, state, area, &def, &pw);
    }

    if state.input_mode == InputMode::Searching {
        render_search_overlay(f, state, area);
    } else if state.input_mode == InputMode::CommandPalette {
        palette::render(f, state, area);
    }
}

fn render_too_small(f: &mut Frame, state: &AppState, area: Rect) {
    use ratatui::widgets::Paragraph;
    let fg = to_color(*state.current_theme.fg());
    let text = Paragraph::new("Terminal too small\nMinimum: 40x8").style(Style::default().fg(fg));
    f.render_widget(text, area);
}

fn render_search_overlay(f: &mut Frame, state: &AppState, area: Rect) {
    use ratatui::widgets::{Block, Borders, Paragraph};
    let fg = to_color(*state.current_theme.fg());
    let bg = to_color(*state.current_theme.bg());
    let accent = to_color(*state.current_theme.accent());
    let search_text = format!("Filter: {}_", state.search_query);
    let overlay = Paragraph::new(search_text)
        .style(Style::default().fg(fg).bg(bg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border_for(state.style.borders))
                .border_style(Style::default().fg(accent))
                .title(Line::from(vec![Span::styled(
                    " Search Processes ",
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                )])),
        );
    let overlay_area = Rect {
        x: area.width.saturating_sub(40) / 2,
        y: area.height.saturating_sub(5) / 2,
        width: 40.min(area.width),
        height: 3.min(area.height),
    };
    f.render_widget(overlay, overlay_area);
}

fn render_fullscreen(f: &mut Frame, state: &mut AppState, area: Rect) {
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);
    let pw = plugin_widgets(state);
    let header_options = current_widget_options(state, "header");
    render_named(f, state, "header", chunks[0], &pw, header_options.as_ref());
    let name = fullscreen_widget_name(state.full_screen_widget);
    let widget_options = current_widget_options(state, name);
    if !render_named(f, state, name, chunks[1], &pw, widget_options.as_ref()) {
        let text = format!("No widget registered for '{name}'");
        let fg = to_color(*state.current_theme.fg());
        let p = ratatui::widgets::Paragraph::new(text).style(Style::default().fg(fg));
        f.render_widget(p, chunks[1]);
    }
}

fn fullscreen_widget_name(w: FullScreenWidget) -> &'static str {
    match w {
        FullScreenWidget::Cpu => "cpu",
        FullScreenWidget::Memory => "memory",
        FullScreenWidget::Storage => "storage",
        FullScreenWidget::Network => "network",
        FullScreenWidget::Processes => "processes",
        FullScreenWidget::DiskIO => "disk_io",
        FullScreenWidget::Gpu => "gpu",
        FullScreenWidget::Battery => "battery",
        FullScreenWidget::None => "cpu",
    }
}

fn render_minimal(f: &mut Frame, state: &mut AppState, area: Rect) {
    use ratatui::widgets::Gauge;

    let bg = to_color(*state.current_theme.bg());

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Min(0),
    ])
    .split(area);

    let pw = plugin_widgets(state);
    let header_options = current_widget_options(state, "header");
    render_named(f, state, "header", chunks[0], &pw, header_options.as_ref());

    let Some(snap) = state.snapshot_cache() else {
        return;
    };
    let cpu_pct = snap.cpus.first().map(|c| c.usage).unwrap_or(0.0);
    let cpu_text = format!(
        "CPU: {:>3.0}%  |  Mem: {:.1}/{:.1}G ({:>3.0}%)",
        cpu_pct,
        snap.memory.used as f64 / 1073741824.0,
        snap.memory.total as f64 / 1073741824.0,
        snap.memory.percent,
    );
    let cpu_gauge = Gauge::default()
        .gauge_style(
            Style::default()
                // Role slot 1 (alert red): CPU metrics (see docs/colors.md).
                .fg(to_color(state.current_theme.palette[1]))
                .bg(bg),
        )
        .percent(cpu_pct as u16)
        .label(cpu_text);
    f.render_widget(cpu_gauge, chunks[1]);

    let mem_pct = snap.memory.percent as u16;
    let mem_text = format!(
        "Mem: {:.1}/{:.1}G ({:>3.0}%)",
        snap.memory.used as f64 / 1073741824.0,
        snap.memory.total as f64 / 1073741824.0,
        snap.memory.percent,
    );
    let mem_gauge = Gauge::default()
        .gauge_style(
            Style::default()
                // Role slot 2 (green): memory metrics (see docs/colors.md).
                .fg(to_color(state.current_theme.palette[2]))
                .bg(bg),
        )
        .percent(mem_pct)
        .label(mem_text);
    f.render_widget(mem_gauge, chunks[2]);

    let processes_options = current_widget_options(state, "processes");
    render_named(
        f,
        state,
        "processes",
        chunks[3],
        &pw,
        processes_options.as_ref(),
    );
}

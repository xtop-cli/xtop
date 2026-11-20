//! `xtop widget scaffold`: generate a fresh single-widget pack crate under
//! `widgets-dev/` (git-ignored), ready to push as its own repo and wire in
//! with `xtop widget install`.
//!
//! The template is a *compiling* skeleton, not a stub: it exports
//! `registry()` per the pack contract and renders a placeholder panel from
//! real snapshot values (process count + CPU/memory gauges) with theme
//! colors only, so a freshly scaffolded crate builds and shows up as a
//! working widget out of the box.

use std::fs;

use super::repo_root;

pub(crate) fn cmd_widget_scaffold(name: &str) -> anyhow::Result<()> {
    let pack_dir = repo_root()
        .join("widgets-dev")
        .join(format!("xtop-widget-{name}"));
    if pack_dir.exists() {
        anyhow::bail!("Widget pack crate already exists at {}", pack_dir.display());
    }
    let src_dir = pack_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    fs::write(pack_dir.join("Cargo.toml"), cargo_toml(name))?;
    fs::write(src_dir.join("lib.rs"), lib_rs(name))?;
    fs::write(pack_dir.join("README.md"), readme(name))?;

    println!("Widget pack scaffold created at {}", pack_dir.display());
    println!("To integrate it into the kernel:");
    println!(
        "  1. `xtop widget install {}` (a local path or a git URL to this",
        pack_dir.display()
    );
    println!("     crate's repo once pushed; works for repos with the crate at");
    println!("     the root or under packs//).");
    println!("  2. Rebuild with the `widget-{name}` feature enabled (add it to");
    println!("     the [features] default list in Cargo.toml, or build with");
    println!("     `--features widget-{name}`), then pick the pack per widget in");
    println!("     config.json (`style.pack` / `style.widgets.<name>.pack`: \"{name}\").");
    Ok(())
}

fn cargo_toml(name: &str) -> String {
    format!(
        r#"[package]
name = "xtop-widget-{name}"
version = "0.1.0"
edition = "2021"
rust-version = "1.87"
license = "GPL-3.0-or-later"
description = "xtop widget pack: {name}"

[dependencies]
ratatui = "0.30.2"
xtop-widget-api = {{ git = "https://github.com/xtop-cli/api" }}
xtop-plugin-api = {{ git = "https://github.com/xtop-cli/api" }}
"#
    )
}

fn readme(name: &str) -> String {
    format!(
        "# xtop-widget-{name}\n\n\
         A single-widget xtop pack scaffolded by `xtop widget scaffold {name}`.\n\n\
         The crate registers one widget named `{name}` and draws a placeholder\n\
         panel from the live snapshot (process count, CPU and memory gauges).\n\
         Replace the body of `render` in `src/lib.rs` with your own drawing;\n\
         keep `registry()` as the pack entry point the kernel calls.\n\n\
         Install it into the kernel with `xtop widget install <repo|path>` and\n\
         enable the `widget-{name}` feature to rebuild xtop with it. Authoring\n\
         guidance lives in the widgets repo docs (`docs/authoring.md`).\n"
    )
}

fn lib_rs(name: &str) -> String {
    let cap: String = {
        let mut chars = name.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        }
    };
    format!(
        r#"//! {cap} widget pack for xtop: a single-widget crate scaffolded by
//! `xtop widget scaffold {name}`.
//!
//! The pack registers one widget named `{name}` (the name layout files and
//! the per-widget style config use) and renders a placeholder panel from
//! real snapshot values — the process count plus CPU/memory gauges — with
//! theme colors only, so the crate compiles and draws out of the box.
//! Replace the body of [`render`] with your own drawing while keeping the
//! `registry()` entry point: the kernel calls it once per pack and resolves
//! `(pack, widget)` by name at render time.

use std::collections::HashMap;
use std::sync::Arc;

use ratatui::layout::Rect;
use ratatui::style::{{Modifier, Style}};
use ratatui::text::{{Line, Span}};
use ratatui::widgets::{{Block, Borders, Paragraph}};
use ratatui::Frame;
use xtop_widget_api::glyph::{{border_for, to_color}};
use xtop_widget_api::{{WidgetRenderer, WidgetState}};

/// The widget name this pack registers (used in layouts and in
/// `style.widgets.<name>.pack`).
pub const WIDGET_NAME: &str = "{name}";

/// Pack entry point: widget name → renderer registry.
pub fn registry() -> HashMap<&'static str, WidgetRenderer> {{
    let mut m: HashMap<&'static str, WidgetRenderer> = HashMap::new();
    m.insert(WIDGET_NAME, Arc::new(render));
    m
}}

/// Draw the placeholder panel: real values, theme colors only.
///
/// Never panic on small areas or an empty snapshot — the kernel renders this
/// before the first tick too (the sample is `None` then).
fn render(f: &mut Frame, state: &dyn WidgetState, area: Rect) {{
    let fg = to_color(*state.theme_fg());
    let bg = to_color(*state.theme_bg());
    let accent = to_color(state.theme_palette()[6]);
    let dim = to_color(state.theme_palette()[8]);

    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            format!(" {{WIDGET_NAME}} "),
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        )]))
        .borders(Borders::ALL)
        .border_set(border_for(state.borders(WIDGET_NAME)))
        .style(Style::default().fg(fg).bg(bg));
    let inner = block.inner(area);
    f.render_widget(block, area);
    if inner.height < 3 || inner.width < 12 {{
        return;
    }}

    let Some(snap) = state.snapshot() else {{
        f.render_widget(
            Paragraph::new(" waiting for the first sample…").style(Style::default().fg(fg).bg(bg)),
            inner,
        );
        return;
    }};

    let cpu_avg = if snap.cpus.is_empty() {{
        0.0
    }} else {{
        snap.cpus.iter().map(|c| c.usage).sum::<f64>() / snap.cpus.len() as f64
    }};
    let alerts = state.alerts();
    let text = vec![
        gauge_line(
            "CPU",
            cpu_avg,
            alerts.cpu_high,
            state.theme_palette(),
            fg,
            dim,
            accent,
        ),
        gauge_line(
            "Mem",
            snap.memory.percent,
            alerts.mem_high,
            state.theme_palette(),
            fg,
            dim,
            accent,
        ),
        Line::from(vec![Span::styled(
            format!(" Procs: {{}}", snap.processes.len()),
            Style::default().fg(fg),
        )]),
    ];
    let p = Paragraph::new(text).style(Style::default().fg(fg).bg(bg));
    f.render_widget(p, inner);
}}

/// One labeled 10-cell gauge line, colored by theme roles only: full cells
/// use the palette role ramp (slot 1 alert, slot 3 warn, slot 2 good), the
/// remainder the dim role (slot 8) — never a fixed color literal.
fn gauge_line(
    label: &str,
    percent: f64,
    alert_at: f64,
    palette: &[[u8; 3]; 16],
    fg: ratatui::style::Color,
    dim: ratatui::style::Color,
    accent: ratatui::style::Color,
) -> Line<'static> {{
    let filled = (percent.clamp(0.0, 100.0) / 10.0).round() as usize;
    let role = if percent >= alert_at {{
        1 // alert role
    }} else if percent >= 50.0 {{
        3 // warn role
    }} else {{
        2 // good role
    }};
    let fill_color = to_color(palette[role]);
    Line::from(vec![
        Span::styled(
            format!(" {{:<4}} {{:>3.0}}% ", label, percent),
            Style::default().fg(fg),
        ),
        Span::styled("█".repeat(filled), Style::default().fg(fill_color)),
        Span::styled("░".repeat(10 - filled), Style::default().fg(dim)),
        Span::styled("  ", Style::default().fg(accent)),
    ])
}}
"#,
        cap = cap,
        name = name
    )
}

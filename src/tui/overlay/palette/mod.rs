//! Command palette widget: themes/layouts quick selection.
//!
//! Kernel-owned chrome (DR-UX3): title and border use the accent role and
//! the border glyph set follows the global `style.borders` choice, so the
//! palette keeps the same look as widget frames and the help overlay.

use crate::core::state::{AppState, PalettePage};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;
use xtop_widget_api::glyph::{border_for, to_color};

/// Centered frame used by the command bar. Kept separate so the renderer and
/// the regression test agree on the cleared region.
pub(crate) fn popup_area(area: Rect) -> Rect {
    let width = (area.width as f64 * 0.6).min(60.0) as u16;
    let height = (area.height as f64 * 0.6).min(30.0) as u16;
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

pub fn render(f: &mut Frame, state: &AppState, area: Rect) {
    let fg = to_color(*state.current_theme.fg());
    let bg = to_color(*state.current_theme.bg());
    let accent = to_color(*state.current_theme.accent());

    let popup = popup_area(area);
    // Clear only the popup: the live dashboard stays visible around it, but
    // never shows through the input row or the list.
    f.render_widget(Clear, popup);
    let title = state.palette.title();
    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            format!(" {title} "),
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        )]))
        .borders(Borders::ALL)
        .border_set(border_for(state.style.borders))
        .border_style(Style::default().fg(accent))
        .style(Style::default().fg(fg).bg(bg));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let hint = match state.palette.page {
        PalettePage::Main => " Command bar · filter: type · ↑↓ navigate · Enter select",
        PalettePage::Plugins => " Plugins · Enter enable/disable · Esc/Bksp back",
        PalettePage::Themes | PalettePage::Layouts => " Filter · Enter select · Esc/Bksp back",
    };
    let input_label = match state.palette.page {
        PalettePage::Main => "Command",
        PalettePage::Themes => "Theme",
        PalettePage::Layouts => "Layout",
        PalettePage::Plugins => "Plugin",
    };

    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).split(inner);
    let search_area = chunks[0];
    let input_text = if state.palette.query.is_empty() {
        format!(" {}_", hint)
    } else {
        format!(" {}: {}", input_label, state.palette.query)
    };
    let input = Paragraph::new(input_text.as_str())
        .style(Style::default().fg(accent).bg(bg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(accent)),
        );
    f.render_widget(input, search_area);

    let list_area = chunks[1];

    let items: Vec<ListItem> = state
        .palette
        .filtered
        .iter()
        .map(|&entry_idx| {
            let entry = &state.palette.entries[entry_idx];
            ListItem::new(entry.label.as_str()).style(Style::default().fg(fg).bg(bg))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(state.palette.selected));

    let list = List::new(items)
        .highlight_style(Style::default().fg(bg).bg(fg).add_modifier(Modifier::BOLD))
        .highlight_symbol("▸ ");
    f.render_stateful_widget(list, list_area, &mut list_state);
}

//! Layout render engine: splits rects and dispatches widget renderers.
//!
//! Widgets live in packs (see the `widgets` repo); the kernel resolves
//! `(pack, name)` at render time from the compile-time pack catalog
//! (`super::pack_table`). Plugin widgets ([`PluginWidget`]) keep precedence
//! over packs and can replace any name. Unknown names are reported once per
//! process (see [`warn_unknown_widget`]).

use super::pack_table::resolve_pack;
use crate::core::state::AppState;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use xtop_layout::{Direction, LayoutArea, LayoutConstraint, LayoutDef, LayoutNode};
use xtop_plugin_api::PluginWidget;
use xtop_widget_api::WidgetRenderer;

/// Resolve the renderer for a widget name following the user's pack choice
/// (`style.pack` global or per-widget `style.widgets.<name>.pack`). Unknown
/// packs and names gracefully fall back to the base pack.
fn resolve(state: &AppState, name: &str) -> Option<&'static WidgetRenderer> {
    resolve_pack(state.style.pack_for(name), name)
}

/// Render a layout definition within a given area.
///
/// `plugin_widgets` is an optional extension from plugins; plugin widgets
/// take precedence over every pack.
pub fn render_layout(
    f: &mut Frame,
    state: &mut AppState,
    area: Rect,
    def: &LayoutDef,
    plugin_widgets: &HashMap<String, PluginWidget>,
) {
    warn_unknown_widgets(state, def, plugin_widgets);
    render_node(f, state, area, &def.root, plugin_widgets);
}

/// Render a single named widget (used by fullscreen and minimal views).
/// Returns false when no renderer is registered for the name.
///
/// `options` are the layout node's display options of the widget instance
/// being rendered (DR-UX1). Pack renderers see them through
/// `WidgetState::widget_options` (widget-api) while they run; plugin widgets
/// render against `HostState` and keep receiving `None` this cycle.
pub fn render_named(
    f: &mut Frame,
    state: &mut AppState,
    name: &str,
    area: Rect,
    plugin_widgets: &HashMap<String, PluginWidget>,
    options: Option<&serde_json::Value>,
) -> bool {
    if let Some(widget) = plugin_widgets.get(name) {
        (widget.render)(f, state, area);
        return true;
    }
    if let Some(render_fn) = resolve(state, name) {
        // Set the active widget options for the duration of this render
        // call, then reset. Every pack render is preceded by a set, so even
        // a panicking renderer cannot leak stale options into a later frame
        // (the render loop does not recover panics).
        state.active_widget_options = options.cloned();
        render_fn(f, state, area);
        state.active_widget_options = None;
        return true;
    }
    false
}

fn render_node(
    f: &mut Frame,
    state: &mut AppState,
    area: Rect,
    node: &LayoutNode,
    plugin_widgets: &HashMap<String, PluginWidget>,
) {
    match node {
        LayoutNode::Widget { name, options } => {
            render_named(f, state, name, area, plugin_widgets, options.as_ref());
        }
        LayoutNode::Split { direction, areas } => {
            if areas.is_empty() {
                return;
            }
            let constraints: Vec<Constraint> = areas.iter().map(to_ratatui_constraint).collect();
            let chunks = match direction {
                Direction::Horizontal => Layout::horizontal(constraints).split(area),
                Direction::Vertical => Layout::vertical(constraints).split(area),
            };
            for (i, chunk) in chunks.iter().enumerate() {
                if i < areas.len() {
                    render_node(f, state, *chunk, &areas[i].node, plugin_widgets);
                }
            }
        }
    }
}

/// Find the display options of the first widget node named `name` in a
/// layout tree (pre-order, depth first), if any.
///
/// Used by render paths that draw a widget by name without walking the tree
/// themselves (fullscreen and minimal views): the widget instance's options
/// come from the layout it would render under, or `None` when the current
/// layout has no such node (the widget then renders with default behavior).
pub fn widget_node_options<'a>(node: &'a LayoutNode, name: &str) -> Option<&'a serde_json::Value> {
    match node {
        LayoutNode::Widget {
            name: widget_name,
            options,
        } => {
            if widget_name == name {
                options.as_ref()
            } else {
                None
            }
        }
        LayoutNode::Split { areas, .. } => areas
            .iter()
            .find_map(|area| widget_node_options(&area.node, name)),
    }
}

fn to_ratatui_constraint(area: &LayoutArea) -> Constraint {
    match area.constraint {
        LayoutConstraint::Length(n) => Constraint::Length(n),
        LayoutConstraint::Percentage(p) => Constraint::Percentage(p),
        LayoutConstraint::Fill => Constraint::Fill(1),
    }
}

// ---------------------------------------------------------------------------
// Unknown-widget reporting (DR-3): a layout referencing a name no pack or
// plugin provides renders an empty area; that is a user-facing mistake, so
// the kernel warns once per widget name per process.
// ---------------------------------------------------------------------------

/// Widget names already reported as unknown this process.
static REPORTED_UNKNOWN_WIDGETS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

/// Emit at most one stderr warning per unknown widget name per process.
fn warn_unknown_widget(layout: &str, name: &str) {
    if report_unknown_widget(name) {
        eprintln!("xtop: layout '{layout}' references unknown widget '{name}'");
    }
}

/// Mark a widget name as reported. Returns `true` on the first call for a
/// given name, `false` for every later call (the one-warning-per-name
/// guarantee). The set never shrinks, so the render path only pays for a
/// name once per process.
fn report_unknown_widget(name: &str) -> bool {
    let reported = REPORTED_UNKNOWN_WIDGETS.get_or_init(|| Mutex::new(HashSet::new()));
    let Ok(mut seen) = reported.lock() else {
        return true;
    };
    seen.insert(name.to_string())
}

/// Walk the layout tree once per frame and warn about leaf names no pack or
/// plugin can render. Cheap (pure name collection over a small tree); the
/// per-name dedup keeps repeated frames silent.
fn warn_unknown_widgets(
    state: &AppState,
    def: &LayoutDef,
    plugin_widgets: &HashMap<String, PluginWidget>,
) {
    fn walk<'a>(node: &'a LayoutNode, names: &mut Vec<&'a str>) {
        match node {
            LayoutNode::Widget { name, .. } => names.push(name),
            LayoutNode::Split { areas, .. } => {
                for area in areas {
                    walk(&area.node, names);
                }
            }
        }
    }

    let mut names = Vec::new();
    walk(&def.root, &mut names);
    for name in names {
        if !plugin_widgets.contains_key(name) && resolve(state, name).is_none() {
            warn_unknown_widget(&def.name, name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{report_unknown_widget, REPORTED_UNKNOWN_WIDGETS};
    use std::collections::HashSet;
    use std::sync::Mutex;

    #[test]
    fn unknown_widget_warns_once_per_name() {
        // Reset the process-wide set so this test is order-independent.
        REPORTED_UNKNOWN_WIDGETS
            .set(Mutex::new(HashSet::new()))
            .ok();
        // First report of a name returns true (a warning is emitted)...
        assert!(report_unknown_widget("ghost"));
        // ...every later report of the same name is silent.
        assert!(!report_unknown_widget("ghost"));
        // A different name still warns.
        assert!(report_unknown_widget("phantom"));
    }
}

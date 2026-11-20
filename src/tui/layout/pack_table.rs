//! Compile-time widget-pack catalog: one source of truth for the packs the
//! render engine enumerates (see [`compiled_packs`]) and `xtop widget list`
//! reports.
//!
//! Every row pairs a Cargo `feature` with the pack `label` users write into
//! `style.pack` / `style.widgets.<name>.pack`:
//!
//! - `feature == "default"` is the always-on base pack (`xtop-widgets`);
//!   it is the fallback for every widget name that no other pack provides.
//! - every other row is gated by the Cargo feature of the same name
//!   (e.g. `widget-blocks` → `xtop-widget-blocks`); the pack is part of the
//!   `packs` map only when that feature is enabled in the current build.
//!
//! `xtop widget install <repo|path>` appends locally installed packs here as
//! a documented self-edit (the same workflow the plugin installer applies to
//! the root `Cargo.toml`): one `PACK_TABLE` row directly above the table
//! marker below and one cfg-gated match arm directly above the arm marker.
//! Both markers are plain comments whose exact text also lives in
//! `src/cli/commands/widget/install.rs` (the installer anchors on that text).
//! Rows and arms are inert data until their feature is enabled, so
//! installing never changes the default build.

use std::collections::HashMap;
use std::sync::OnceLock;

use xtop_widget_api::WidgetRenderer;

/// One catalog row: `feature` gates the pack crate (`"default"` = always
/// on), `label` is the pack name used in the user config.
pub(crate) struct PackRow {
    pub(crate) feature: &'static str,
    pub(crate) label: &'static str,
}

/// Shipped packs in precedence order (the base pack first; it is the
/// fallback for unknown pack choices and widget names).
pub(crate) const PACK_TABLE: &[PackRow] = &[
    PackRow {
        feature: "default",
        label: "default",
    },
    // (xtop widget install appends installed-pack rows above this line.)
    PackRow {
        feature: "widget-blocks",
        label: "blocks",
    },
];

/// One compiled-in widget pack: the label from [`PACK_TABLE`] plus the
/// registry of renderers its crate exposes.
pub(crate) struct Pack {
    name: &'static str,
    renderers: &'static HashMap<&'static str, WidgetRenderer>,
}

static BASE_PACK: OnceLock<HashMap<&'static str, WidgetRenderer>> = OnceLock::new();
#[cfg(feature = "widget-blocks")]
static BLOCKS_PACK: OnceLock<HashMap<&'static str, WidgetRenderer>> = OnceLock::new();

/// The packs compiled into this binary, in [`PACK_TABLE`] order.
///
/// Each optional row is resolved by a compile-time arm below (one per pack
/// crate the kernel can link); rows whose feature is off in this build are
/// skipped. The base pack row is unconditional.
pub(crate) fn compiled_packs() -> &'static [Pack] {
    static PACKS: OnceLock<Vec<Pack>> = OnceLock::new();
    PACKS.get_or_init(|| {
        let mut v: Vec<Pack> = Vec::new();
        for row in PACK_TABLE {
            if row.feature == "default" {
                v.push(Pack {
                    name: row.label,
                    renderers: BASE_PACK.get_or_init(xtop_widgets::registry),
                });
            } else {
                push_optional_pack(&mut v, row);
            }
        }
        v
    })
}

/// Resolve one optional table row to its crate registry when the row's Cargo
/// feature is compiled in.
// The Vec is required by the cfg-gated `push` arms; with no optional pack
// compiled in the parameter would otherwise be a needless-`Vec` lint.
#[cfg_attr(
    not(feature = "widget-blocks"),
    allow(unused_variables, clippy::ptr_arg)
)]
fn push_optional_pack(v: &mut Vec<Pack>, row: &PackRow) {
    match row.feature {
        // (xtop widget install appends installed-pack arms above this line.)
        #[cfg(feature = "widget-blocks")]
        "widget-blocks" => v.push(Pack {
            name: row.label,
            renderers: BLOCKS_PACK.get_or_init(xtop_widget_blocks::registry),
        }),
        _ => {}
    }
}

/// The pack that owns the renderer for a widget name, or `None` when no
/// compiled pack provides it. `pack_label` is the user's chosen pack for the
/// name (from `style.pack_for`); when it does not resolve, the base pack is
/// the fallback, mirroring the pre-catalog precedence.
pub(crate) fn resolve_pack(
    pack_label: Option<&str>,
    name: &str,
) -> Option<&'static WidgetRenderer> {
    let packs = compiled_packs();
    if let Some(pack_name) = pack_label {
        if let Some(pack) = packs.iter().find(|p| p.name == pack_name) {
            if let Some(r) = pack.renderers.get(name) {
                return Some(r);
            }
        }
    }
    packs
        .iter()
        .find(|p| p.name == "default")
        .and_then(|p| p.renderers.get(name))
}

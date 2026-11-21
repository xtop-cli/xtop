//! Layout area: the engine that maps layout definitions onto widget
//! renderers, plus the built-in widget registry.
//!
//! `pack_table` is the single source of truth for the compiled widget
//! packs (`PACK_TABLE` rows + the cfg-gated arms that link each pack crate);
//! the engine resolves `(pack, name)` against it.

mod engine;
pub(crate) mod pack_table;

pub use engine::*;

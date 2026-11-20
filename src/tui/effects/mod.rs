//! Optional frame-effect host (feature `effects`).
//!
//! Effects (see `xtop-effect-api`) transform the fully rendered frame
//! buffer after layout and before the terminal flush. The kernel hosts at
//! most one active effect, named by the optional persisted `effect` config
//! key, and drives it with the time elapsed since the run loop started.
//!
//! When the feature is off this module still compiles (same type, idle
//! no-op), so the run loop stays feature-agnostic: default builds carry
//! zero extra dependencies and zero runtime cost.

#[cfg(not(feature = "effects"))]
mod off;
#[cfg(feature = "effects")]
mod on;

#[cfg(not(feature = "effects"))]
pub use off::EffectHost;
#[cfg(feature = "effects")]
pub use on::EffectHost;

//! Idle effect host for builds without the `effects` feature.
//!
//! Same public surface as the feature-on host (see the parent module doc),
//! but zero-sized: the config key is ignored, `apply` is a no-op and no
//! effect dependency is compiled in.

/// Idle frame-effect host (feature `effects` not compiled in).
pub struct EffectHost;

impl EffectHost {
    /// Idle host: without the feature there is no effect to activate, so
    /// the config value is deliberately ignored.
    pub fn from_config(_name: Option<&str>) -> Self {
        Self
    }

    /// No-op: the feature is off, the frame is passed through untouched.
    pub fn apply(&mut self, _buffer: &mut ratatui::buffer::Buffer) {}
}

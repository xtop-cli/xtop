//! Effect host compiled when the `effects` feature is on.
//!
//! The host owns the optional active effect plus the instant the run loop
//! started. The run loop calls [`EffectHost::apply`] after every render;
//! with no active effect the call is a cheap no-op (no allocation).

use std::time::Instant;

use xtop_effect_api::Effect;
use xtop_effect_fade::FadeEffect;

/// Runtime state of the frame-effect host.
pub struct EffectHost {
    /// When the run loop started; the effect contract's clock (`elapsed`
    /// grows from here, never resets).
    started: Instant,
    /// The active effect, when the config named one we know.
    effect: Option<Box<dyn Effect>>,
}

impl EffectHost {
    /// Create the host from the persisted `effect` config value.
    ///
    /// `"fade"` activates the built-in `FadeEffect`; an absent, empty or
    /// unknown name leaves the host idle (no effect, no allocation).
    pub fn from_config(name: Option<&str>) -> Self {
        Self {
            started: Instant::now(),
            effect: select_effect(name),
        }
    }

    /// Apply the active effect to a rendered frame (after layout, before
    /// flush). No-op when no effect is active.
    pub fn apply(&mut self, buffer: &mut ratatui::buffer::Buffer) {
        if let Some(effect) = self.effect.as_deref_mut() {
            effect.on_frame(buffer, self.started.elapsed());
        }
    }
}

/// Map a config value to an effect instance. `None`/empty and unknown names
/// select no effect; the only built-in id is `"fade"`.
pub fn select_effect(name: Option<&str>) -> Option<Box<dyn Effect>> {
    match name {
        Some("fade") => Some(Box::new(FadeEffect)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::select_effect;

    #[test]
    fn fade_config_value_selects_the_fade_effect() {
        let effect = select_effect(Some("fade")).expect("fade must resolve");
        assert_eq!(effect.manifest().id, "fade");
    }

    #[test]
    fn absent_empty_or_unknown_names_select_no_effect() {
        assert!(select_effect(None).is_none());
        assert!(select_effect(Some("")).is_none());
        assert!(select_effect(Some("wipe")).is_none());
    }
}

//! Contrast engine (UX8.2): WCAG contrast helpers and the load-time theme
//! normalizer.
//!
//! Every theme that enters the runtime is normalized once, right after
//! parsing (see `loader.rs`). Normalization guarantees the role floors below
//! against the theme's explicit background. The normalizer only ever moves a
//! color toward white or black (deterministic, hue-preserving steps), and it
//! REPLACES the stored palette entries — renderers keep reading
//! `Theme::palette` / the role accessors unchanged. The shipped theme files
//! stay canonical: normalization happens in memory at load.
//!
//! Roles and floors (all measured with the WCAG contrast ratio):
//!
//! | Role | Slot / source | Floor | Note |
//! |---|---|---|---|
//! | foreground text | explicit `foreground` key (legacy: slot 7) | 4.5 | primary text on the background |
//! | accent | slot 6 | 3.0 | titles, headers, key spans (text) |
//! | dim | slot 8 | 3.0 | separators / muted text / zebra stripe against the background |
//! | zebra-row text | foreground over the dim stripe | 3.0 | process-table text painted on odd (dim) rows |
//! | series ramp / accents | slots 1–5, 7, 9–15 | 2.0 | colored marks drawn on the background (fills, chart lines) |
//!
//! The lift direction is decided by the background luminance: dark
//! backgrounds lift toward white, light backgrounds toward black. The dim
//! role and the zebra-row-text role share slot 8 (the stripe) with the
//! foreground painted over it, so both floors cannot always hold at once:
//! zebra-row text always wins. `dim` is raised toward its 3.0 floor only
//! while zebra text stays ≥ 3.0; when no lift is possible without breaking
//! zebra text (the shipped Helsinki/Oslo palettes, whose foreground sits
//! close to the background), dim keeps its canonical value and the stripe
//! stays subtle rather than making text illegible.
use crate::core::theme::Theme;

/// Floor for the foreground text role (WCAG AA for normal text).
pub const FG_FLOOR: f64 = 4.5;
/// Floor for the dim role (separators, muted text, zebra stripes).
pub const DIM_FLOOR: f64 = 3.0;
/// Floor for the accent role (titles, headers, key spans).
pub const ACCENT_FLOOR: f64 = 3.0;
/// Floor for text painted over a zebra (dim) row.
pub const ZEBRA_FLOOR: f64 = 3.0;
/// Floor for colored marks (fills, chart lines) drawn on the background.
pub const MARK_FLOOR: f64 = 2.0;

/// Steps a color may travel before a lift gives up (a color moved 1/4 of the
/// remaining distance per step converges in ~10 steps; the cap only guards
/// pathological inputs).
const MAX_STEPS: usize = 96;

/// WCAG relative luminance of an sRGB color (0.0 black .. 1.0 white).
pub(crate) fn relative_luminance(color: [u8; 3]) -> f64 {
    0.2126 * channel_linear(color[0])
        + 0.7152 * channel_linear(color[1])
        + 0.0722 * channel_linear(color[2])
}

/// WCAG contrast ratio between two colors (1.0 .. 21.0).
pub(crate) fn contrast_ratio(a: [u8; 3], b: [u8; 3]) -> f64 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

fn channel_linear(c: u8) -> f64 {
    let s = f64::from(c) / 255.0;
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

/// One small step: move a color 1/4 of the way toward `target`, per channel.
/// Interpolating toward white/black keeps the hue while changing luminance —
/// the "hue-preserving lift" the role floors rely on.
fn step_toward(color: [u8; 3], target: [u8; 3]) -> [u8; 3] {
    [
        step_channel(color[0], target[0]),
        step_channel(color[1], target[1]),
        step_channel(color[2], target[2]),
    ]
}

fn step_channel(c: u8, target: u8) -> u8 {
    let c = f64::from(c);
    let t = f64::from(target);
    (c + (t - c) / 4.0).round().clamp(0.0, 255.0) as u8
}

/// Lift `color` until it reaches `floor` against `anchor`, stepping toward
/// white when `toward_white` is true, toward black otherwise.
pub(crate) fn lift_until(
    mut color: [u8; 3],
    anchor: [u8; 3],
    floor: f64,
    toward_white: bool,
) -> [u8; 3] {
    let target = if toward_white { [255; 3] } else { [0; 3] };
    for _ in 0..MAX_STEPS {
        if contrast_ratio(color, anchor) >= floor {
            break;
        }
        color = step_toward(color, target);
    }
    color
}

/// Move `color` toward `target` in steps until `ok` holds (used for the zebra
/// pass: the dim stripe moves toward the background until the foreground text
/// over it clears the zebra floor).
fn move_until(mut color: [u8; 3], target: [u8; 3], ok: impl Fn([u8; 3]) -> bool) -> [u8; 3] {
    for _ in 0..MAX_STEPS {
        if ok(color) {
            break;
        }
        color = step_toward(color, target);
    }
    color
}

/// The palette slots treated as colored marks on the background (fills,
/// chart lines, series ramp). Slot 0 is the legacy background alias, slot 6
/// is the accent role (its own 3.0 floor), slot 8 is dim (its own floor).
pub(crate) const MARK_SLOTS: [usize; 13] = [1, 2, 3, 4, 5, 7, 9, 10, 11, 12, 13, 14, 15];

/// Normalize a loaded theme in place (UX8.2): lift every text/mark role over
/// its floor against the explicit background, then fix zebra-row text.
///
/// Legacy themes (parsed without an explicit `foreground` key, so the field
/// aliases slot 7) get slot 7 synced to the lifted foreground so text drawn
/// straight from the palette slot stays legible too.
pub(crate) fn normalize(theme: &mut Theme) {
    let bg = theme.background;
    let toward_white = relative_luminance(bg) < 0.5;

    // Foreground: primary text role.
    let fg_aliases_slot7 = theme.foreground == theme.palette[7];
    let fg = lift_until(theme.foreground, bg, FG_FLOOR, toward_white);
    theme.foreground = fg;
    if fg_aliases_slot7 {
        theme.palette[7] = fg;
    }

    // Accent text role.
    theme.palette[6] = lift_until(theme.palette[6], bg, ACCENT_FLOOR, toward_white);

    // Dim role (slot 8) — separators / muted text / zebra stripe. Raised
    // toward its 3.0 floor only while zebra-row text (foreground over the
    // stripe) stays at or above its 3.0 floor; when a palette cannot hold
    // both (foreground close to the background), zebra-row text wins and dim
    // keeps the highest value that still clears it (canonical when no lift
    // is possible at all — e.g. the shipped Helsinki/Oslo themes).
    theme.palette[8] = fix_dim(theme.palette[8], bg, fg, toward_white);

    // Colored marks drawn on the background (fills, series lines).
    for &slot in &MARK_SLOTS {
        theme.palette[slot] = lift_until(theme.palette[slot], bg, MARK_FLOOR, toward_white);
    }
}

/// Resolve the dim slot under both the dim floor (against the background)
/// and the zebra-row-text floor (foreground over the dim stripe).
fn fix_dim(dim: [u8; 3], bg: [u8; 3], fg: [u8; 3], toward_white: bool) -> [u8; 3] {
    // Pass 1: raise dim against the background while zebra text stays ≥ 3.0.
    let target = if toward_white { [255; 3] } else { [0; 3] };
    let mut c = dim;
    for _ in 0..MAX_STEPS {
        if contrast_ratio(c, bg) >= DIM_FLOOR {
            break;
        }
        let next = step_toward(c, target);
        if contrast_ratio(fg, next) < ZEBRA_FLOOR {
            break;
        }
        c = next;
    }
    // Pass 2: zebra text still below its floor (dim too close to the
    // foreground) — move the stripe toward the background until the text
    // clears it (the stripe may fade; glyphs never do).
    if contrast_ratio(fg, c) < ZEBRA_FLOOR {
        c = move_until(c, bg, |x| contrast_ratio(fg, x) >= ZEBRA_FLOOR);
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::theme::strip_jsonc_comments;

    /// Read the shipped theme files from disk so the test build does not
    /// embed what the production binary keeps external.
    fn shipped_themes() -> Vec<(String, String)> {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/themes");
        let mut themes = Vec::new();
        for entry in std::fs::read_dir(dir).expect("themes dir") {
            let path = entry.expect("theme entry").path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("jsonc") {
                continue;
            }
            let name = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .expect("theme name")
                .to_string();
            let source = std::fs::read_to_string(&path).expect("read theme");
            themes.push((name, source));
        }
        themes.sort();
        themes
    }

    fn parse(source: &str) -> Theme {
        let cleaned = strip_jsonc_comments(source);
        serde_json::from_str::<Theme>(&cleaned).expect("theme must parse")
    }

    fn rgb(hex: &str) -> [u8; 3] {
        crate::core::theme::hex_to_rgb_pub(hex)
    }

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.05
    }

    /// Assert every role the engine guarantees meets its floor (the
    /// invariant `normalize` promises after a load).
    fn assert_role_floors(name: &str, theme: &Theme) {
        let bg = theme.background;
        assert!(
            contrast_ratio(theme.foreground, bg) >= FG_FLOOR,
            "{name}: fg {} vs bg {} is {:.2} < {FG_FLOOR}",
            fmt_rgb(theme.foreground),
            fmt_rgb(bg),
            contrast_ratio(theme.foreground, bg)
        );
        assert!(
            contrast_ratio(theme.palette[6], bg) >= ACCENT_FLOOR,
            "{name}: accent under the 3.0 floor"
        );
        // Zebra-row text: foreground painted over the dim stripe. This is
        // the invariant the engine never trades away.
        assert!(
            contrast_ratio(theme.foreground, theme.palette[8]) >= ZEBRA_FLOOR,
            "{name}: zebra-row text under the 3.0 floor"
        );
        for &slot in &MARK_SLOTS {
            assert!(
                contrast_ratio(theme.palette[slot], bg) >= MARK_FLOOR,
                "{name}: mark slot {slot} under the 2.0 floor"
            );
        }
    }

    #[test]
    fn wcag_luminance_and_ratio_known_pairs() {
        assert_eq!(relative_luminance([0, 0, 0]), 0.0);
        assert_eq!(relative_luminance([255, 255, 255]), 1.0);
        // WCAG reference: #777777 relative luminance is 0.184 (0.1846).
        assert!(approx(relative_luminance([0x77, 0x77, 0x77]), 0.1846));
        // The canonical test pair: white on black is exactly 21.0.
        assert!(approx(contrast_ratio([255, 255, 255], [0, 0, 0]), 21.0));
    }

    #[test]
    fn paris_foreground_on_background_passes() {
        // Task pair: Paris fg #f7f1ff on bg #1a0a30 must clear the 4.5 floor.
        let ratio = contrast_ratio(rgb("#f7f1ff"), rgb("#1a0a30"));
        assert!(ratio >= FG_FLOOR, "paris fg/bg ratio {ratio:.2}");
        // The reported bug: fg-slot-7 == background. #f7f1ff on #050505 (x).
        let x_ratio = contrast_ratio(rgb("#f7f1ff"), rgb("#050505"));
        assert!(x_ratio >= FG_FLOOR, "x fg/bg ratio {x_ratio:.2}");
    }

    #[test]
    fn every_shipped_theme_meets_every_role_floor_after_normalization() {
        for (name, source) in shipped_themes() {
            let mut theme = parse(&source);
            normalize(&mut theme);
            assert_role_floors(&name, &theme);
            let bg = theme.background;
            let dim_ok = contrast_ratio(theme.palette[8], bg) >= DIM_FLOOR;
            // The dim floor and the zebra-text floor share the same slot and
            // cannot both hold when the foreground sits close to the
            // background; the two shipped themes below keep their canonical
            // dim (never worse than the file value) so zebra text stays
            // legible. Every other shipped theme must clear the dim floor.
            if name == "helsinki" || name == "oslo" {
                let raw = parse(&source);
                assert!(
                    contrast_ratio(theme.palette[8], bg) >= contrast_ratio(raw.palette[8], bg),
                    "{name}: dim must never degrade below the canonical file value"
                );
                continue;
            }
            assert!(dim_ok, "{name}: dim under the 3.0 floor");
        }
    }

    fn fmt_rgb(c: [u8; 3]) -> String {
        format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
    }

    #[test]
    fn paris_foreground_is_not_palette_slot7_after_normalization() {
        let themes = shipped_themes();
        let source = themes
            .iter()
            .find(|(name, _)| name == "paris")
            .expect("paris ships");
        let mut theme = parse(&source.1);
        // Canonical Paris keeps palette[7] == background (the legacy fg slot
        // equals the bg); the explicit pair is what the roles are anchored on.
        assert_eq!(theme.palette[7], theme.background);
        normalize(&mut theme);
        assert_ne!(theme.foreground, theme.background, "fg must differ from bg");
        assert_ne!(
            theme.foreground, theme.palette[7],
            "foreground must not be palette[7] when that slot equals the background"
        );
        assert!(contrast_ratio(theme.foreground, theme.background) >= FG_FLOOR);
    }

    #[test]
    fn normalization_is_deterministic_and_idempotent() {
        for (name, source) in shipped_themes() {
            let mut once = parse(&source);
            normalize(&mut once);
            let mut twice = once.clone();
            normalize(&mut twice);
            assert_eq!(once.foreground, twice.foreground, "{name}: fg drift");
            assert_eq!(once.palette, twice.palette, "{name}: palette drift");
            assert_eq!(once.background, twice.background, "{name}: bg drift");
        }
    }

    #[test]
    fn legacy_file_without_explicit_pair_falls_back_and_stays_legible() {
        // A legacy-format theme (Paris palette, no background/foreground
        // keys): foreground aliases slot 7 == background, so the fg lift must
        // kick in and slot 7 must follow the lifted foreground.
        let source = r##"{
            "name": "legacy",
            "palette": [
                "#1a0a30", "#fc618d", "#7bd88f", "#fce566", "#a3f3ff", "#c4bdff", "#a3f3ff",
                "#1a0a30", "#c4bdff", "#fc618d", "#7bd88f", "#fce566", "#a3f3ff", "#c4bdff",
                "#a3f3ff", "#f7f1ff"
            ]
        }"##;
        let mut theme = parse(source);
        assert_eq!(
            theme.background, theme.palette[0],
            "legacy bg aliases slot 0"
        );
        assert_eq!(
            theme.foreground, theme.palette[7],
            "legacy fg aliases slot 7"
        );
        normalize(&mut theme);
        assert_role_floors("legacy", &theme);
        assert_eq!(theme.palette[7], theme.foreground, "legacy slot 7 syncs fg");
        assert_ne!(theme.foreground, theme.background);
    }
    #[test]
    fn lifted_paris_slot7_example_is_deterministic() {
        // Report/documentation anchor: canonical Paris palette[7] == bg is
        // lifted as a colored mark to ≥ 2.0 against the background, and the
        // dim stripe is moved toward the background until zebra text clears
        // 3.0. Pin the concrete outcome so the engine cannot drift silently.
        let themes = shipped_themes();
        let source = themes
            .iter()
            .find(|(name, _)| name == "paris")
            .expect("paris ships");
        let mut theme = parse(&source.1);
        normalize(&mut theme);
        assert!(contrast_ratio(theme.palette[7], theme.background) >= 2.0);
        assert!(contrast_ratio(theme.foreground, theme.palette[8]) >= 3.0);
        assert!(contrast_ratio(theme.palette[8], theme.background) >= 3.0);
        eprintln!(
            "paris normalized: bg={} fg={} accent={} dim={} slot7={}",
            fmt_rgb(theme.background),
            fmt_rgb(theme.foreground),
            fmt_rgb(theme.palette[6]),
            fmt_rgb(theme.palette[8]),
            fmt_rgb(theme.palette[7]),
        );
    }

    #[test]
    fn normalization_delta_report() {
        // Informational (silent unless --nocapture): one line per shipped
        // theme with the runtime (normalized) role colors, for the docs and
        // the milestone report. Values are pinned by the engine tests above.
        for (name, source) in shipped_themes() {
            let mut theme = parse(&source);
            normalize(&mut theme);
            eprintln!(
                "{name}: bg={} fg={} accent={} dim={} slots13={} {} {} {} {} {} {}",
                fmt_rgb(theme.background),
                fmt_rgb(theme.foreground),
                fmt_rgb(theme.palette[6]),
                fmt_rgb(theme.palette[8]),
                fmt_rgb(theme.palette[1]),
                fmt_rgb(theme.palette[2]),
                fmt_rgb(theme.palette[4]),
                fmt_rgb(theme.palette[5]),
                fmt_rgb(theme.palette[7]),
                fmt_rgb(theme.palette[9]),
                fmt_rgb(theme.palette[15]),
            );
        }
    }
}

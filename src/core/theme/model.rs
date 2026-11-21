//! Theme data model and its custom (hex-string) deserializer.
//!
//! Theme files (UX8.1 format) carry an explicit `background`/`foreground`
//! pair plus the 16-slot `palette`; files without the explicit pair (legacy
//! third-party themes) fall back to slot 0 / slot 7. Themes are contrast-
//! normalized when loaded (see `contrast.rs`): the shipped file always stays
//! canonical, the in-memory `Theme` is the guaranteed-legible runtime view.
use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Serialize)]
pub struct Theme {
    pub name: String,
    /// Explicit background color — the screen/frame background role. Files
    /// without this key (legacy format) fall back to `palette[0]`.
    pub background: [u8; 3],
    /// Explicit foreground color — the primary-text role. Files without this
    /// key (legacy format) fall back to `palette[7]`.
    pub foreground: [u8; 3],
    pub palette: [[u8; 3]; 16],
}

impl Theme {
    /// Background role: the explicit `background` key (legacy fallback:
    /// palette slot 0). Never lifted by the contrast normalizer — every other
    /// role is measured against it.
    pub fn bg(&self) -> &[u8; 3] {
        &self.background
    }

    /// Foreground role: the explicit `foreground` key (legacy fallback:
    /// palette slot 7). Guaranteed ≥ 4.5:1 contrast against the background
    /// after load-time normalization.
    pub fn fg(&self) -> &[u8; 3] {
        &self.foreground
    }

    /// Accent role: palette slot 6 — titles, key spans, selected/highlight
    /// text, overlay borders. Matches the role table in `docs/colors.md`
    /// (the same slot the widget packs use for process headers/selection).
    /// Normalized to ≥ 3.0:1 against the background at load.
    pub fn accent(&self) -> &[u8; 3] {
        &self.palette[6]
    }

    /// Dim role: palette slot 8 — separators, secondary/muted text, zebra
    /// stripe backgrounds (widgets use it for odd process rows). Matches the
    /// role table in `docs/colors.md`. Normalized at load: ≥ 3.0:1 against
    /// the background, with zebra-row text (foreground over the stripe)
    /// kept ≥ 3.0:1 — see `contrast.rs` for the trade when a palette cannot
    /// satisfy both.
    pub fn dim(&self) -> &[u8; 3] {
        &self.palette[8]
    }
}

// Custom deserializer that reads palette as [String; 16] (hex) and converts to [[u8; 3]; 16].
// `background`/`foreground` are optional: absent keys (legacy files) fall back
// to palette[0]/palette[7] once the palette has been parsed.
impl<'de> Deserialize<'de> for Theme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Name,
            Background,
            Foreground,
            Palette,
        }

        struct ThemeVisitor;
        impl<'de> Visitor<'de> for ThemeVisitor {
            type Value = Theme;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Theme")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Theme, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name = None;
                let mut background: Option<String> = None;
                let mut foreground: Option<String> = None;
                let mut palette: Option<[String; 16]> = None;

                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value::<String>()?);
                        }
                        Field::Background => {
                            if background.is_some() {
                                return Err(de::Error::duplicate_field("background"));
                            }
                            background = Some(map.next_value::<String>()?);
                        }
                        Field::Foreground => {
                            if foreground.is_some() {
                                return Err(de::Error::duplicate_field("foreground"));
                            }
                            foreground = Some(map.next_value::<String>()?);
                        }
                        Field::Palette => {
                            if palette.is_some() {
                                return Err(de::Error::duplicate_field("palette"));
                            }
                            palette = Some(map.next_value::<[String; 16]>()?);
                        }
                    }
                }

                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let palette_str = palette.ok_or_else(|| de::Error::missing_field("palette"))?;

                let mut palette = [[0u8; 3]; 16];
                for (i, hex) in palette_str.iter().enumerate() {
                    palette[i] = xtop_plugin_api::hex_to_rgb(hex);
                }

                // Legacy files (no explicit pair) anchor on slot 0 / slot 7.
                let background = match background {
                    Some(hex) => xtop_plugin_api::hex_to_rgb(&hex),
                    None => palette[0],
                };
                let foreground = match foreground {
                    Some(hex) => xtop_plugin_api::hex_to_rgb(&hex),
                    None => palette[7],
                };

                Ok(Theme {
                    name,
                    background,
                    foreground,
                    palette,
                })
            }
        }

        deserializer.deserialize_struct(
            "Theme",
            &["name", "background", "foreground", "palette"],
            ThemeVisitor,
        )
    }
}

pub(crate) fn hex_to_rgb_pub(hex: &str) -> [u8; 3] {
    xtop_plugin_api::hex_to_rgb(hex)
}

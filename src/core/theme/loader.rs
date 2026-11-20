//! Theme loading: embedded default theme and user themes.
//!
//! Every theme is contrast-normalized right after parsing (UX8.2): the
//! shipped/user files always stay canonical, the in-memory `Theme` is the
//! guaranteed-legible runtime view (see `contrast.rs` for the role floors).
use crate::core::theme::Theme;
use std::fs;
use std::path::Path;

fn make_theme(name: &str, background: &str, foreground: &str, colors: [&str; 16]) -> Theme {
    let mut palette = [[0u8; 3]; 16];
    for (i, h) in colors.iter().enumerate() {
        palette[i] = crate::core::theme::hex_to_rgb_pub(h);
    }
    Theme {
        name: name.to_string(),
        background: crate::core::theme::hex_to_rgb_pub(background),
        foreground: crate::core::theme::hex_to_rgb_pub(foreground),
        palette,
    }
}

fn default_theme() -> Theme {
    // Canonical "x" (owner palette source; mirrors assets/themes/x.jsonc).
    make_theme(
        "x",
        "#050505",
        "#f7f1ff",
        [
            "#0a0a0a", "#fc618d", "#7bd88f", "#fce566", "#fd9353", "#948ae3", "#5ad4e6", "#f7f1ff",
            "#0f0f0f", "#fc618d", "#7bd88f", "#fce566", "#fd9353", "#948ae3", "#5ad4e6", "#f7f1ff",
        ],
    )
}

pub(crate) fn strip_jsonc_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '/' && i + 1 < chars.len() {
            if chars[i + 1] == '/' {
                i += 2;
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }
            if chars[i + 1] == '*' {
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Parse one JSONC theme source and normalize it.
///
/// Used by the loader and by the asset downloader before installing a theme.
pub fn parse_theme_source(source: &str) -> Option<Theme> {
    let cleaned = strip_jsonc_comments(source);
    let mut theme = serde_json::from_str::<Theme>(&cleaned).ok()?;
    crate::core::theme::contrast::normalize(&mut theme);
    Some(theme)
}

fn load_theme_from_file(path: &Path) -> Option<Theme> {
    let data = fs::read_to_string(path).ok()?;
    parse_theme_source(&data)
}

fn load_themes_from_dir(dir: &Path) -> Vec<Theme> {
    if !dir.exists() {
        return vec![];
    }
    let mut themes = vec![];
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str());
            if ext == Some("json") || ext == Some("jsonc") {
                if let Some(theme) = load_theme_from_file(&path) {
                    themes.push(theme);
                }
            }
        }
    }
    themes
}

/// Where user themes live: the platform-aware config dir (same tree as
/// `config.json` and the layouts), joined with `themes`. This keeps macOS
/// (`~/Library/Application Support/xtop/themes`) and Windows
/// (`%APPDATA%\xtop\themes`) consistent with the rest of the app instead of
/// hard-coding the XDG `~/.config` layout.
pub fn themes_dir() -> std::path::PathBuf {
    crate::core::config::config_dir().join("themes")
}

pub fn load_all_themes() -> Vec<Theme> {
    // Defaults first (index 0 = "x", stable palette position), then user
    // files: a user theme reusing a default name overrides it in place
    // (parity with layouts); new names are appended. Every theme is
    // contrast-normalized at load.
    let mut themes = vec![default_theme()];
    crate::core::theme::contrast::normalize(&mut themes[0]);
    for t in load_themes_from_dir(&themes_dir()) {
        if let Some(slot) = themes.iter_mut().find(|existing| existing.name == t.name) {
            *slot = t;
        } else {
            themes.push(t);
        }
    }
    themes
}

#[cfg(test)]
pub fn builtin_themes() -> Vec<Theme> {
    let mut themes = vec![default_theme()];
    crate::core::theme::contrast::normalize(&mut themes[0]);
    themes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let t = default_theme();
        assert_eq!(t.name, "x");
        // Explicit background/foreground pair (canonical "x").
        assert_eq!(t.bg(), &[5, 5, 5]);
        assert_eq!(t.fg(), &[247, 241, 255]);
        assert_eq!(t.palette[0], [10, 10, 10]);
    }

    #[test]
    fn test_builtin_themes_count() {
        let themes = builtin_themes();
        assert_eq!(themes.len(), 1);
    }

    #[test]
    fn test_strip_line_comment() {
        let input = "{\"name\": \"test\" // comment\n}";
        let result = strip_jsonc_comments(input);
        assert_eq!(result, "{\"name\": \"test\" \n}");
    }

    #[test]
    fn test_strip_block_comment() {
        let input = "{\"name\": /* comment */ \"test\"}";
        let result = strip_jsonc_comments(input);
        assert_eq!(result, "{\"name\":  \"test\"}");
    }

    #[test]
    fn test_strip_no_comments() {
        let input = "{\"name\": \"test\"}";
        let result = strip_jsonc_comments(input);
        assert_eq!(result, "{\"name\": \"test\"}");
    }

    #[test]
    fn test_hex_to_rgb() {
        let result = crate::core::theme::hex_to_rgb_pub("#ff0000");
        assert_eq!(result, [255, 0, 0]);
    }
}

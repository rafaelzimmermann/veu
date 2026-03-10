use iced::Color;

/// UI colour palette for veu.
///
/// Theme files live in `~/.config/veu/theme.conf`.
/// Each line is `key = #RRGGBB` or `key = #RRGGBBAA`.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// Popup window background.
    pub background: Color,
    /// Primary text colour.
    pub text: Color,
    /// Accent — active slider rail and active mute button.
    pub accent: Color,
    /// Mute button background when unmuted.
    pub button_inactive: Color,
    /// Slider rail to the right of the handle.
    pub slider_inactive: Color,
    /// Slider handle.
    pub handle: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background:      hex("#1e1e2eee"),
            text:            hex("#ffffff"),
            accent:          hex("#ff9500"),
            button_inactive: hex("#4d4d59"),
            slider_inactive: hex("#ffffff33"),
            handle:          hex("#ffffff"),
        }
    }
}

impl Theme {
    /// Load the active theme.
    ///
    /// Resolution order (later steps override earlier ones):
    /// 1. `Default` — compiled-in values.
    /// 2. `~/.config/veu/theme.conf` — user overrides.
    /// 3. `~/.config/veu/themes/<name>.conf` — if `~/.config/veu/current-theme`
    ///    exists and names a bundled or user-installed theme, that file wins.
    pub fn load() -> Self {
        // Start from user's theme.conf (if any), otherwise the compiled default.
        let mut theme = config_path("theme.conf")
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|c| Self::parse(&c))
            .unwrap_or_default();

        // Override with a named theme if current-theme is set.
        if let Some(name) = config_path("current-theme")
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            if let Some(named) = config_path(&format!("themes/{}.conf", name))
                .and_then(|p| Theme::from_file(&p))
            {
                theme = named;
            }
        }

        theme
    }

    /// Load a theme from an arbitrary `.conf` file. Returns `None` if unreadable.
    pub fn from_file(path: &std::path::Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        Some(Self::parse(&content))
    }

    fn parse(content: &str) -> Self {
        let mut theme = Self::default();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, val)) = line.split_once('=') else { continue };
            theme.apply_key(key.trim(), val.trim().trim_matches('"'));
        }
        theme
    }

    pub fn apply_key(&mut self, key: &str, value: &str) {
        let Some(color) = parse_color(value) else { return };
        match key {
            "background"      => self.background      = color,
            "text"            => self.text            = color,
            "accent"          => self.accent          = color,
            "button_inactive" => self.button_inactive = color,
            "slider_inactive" => self.slider_inactive = color,
            "handle"          => self.handle          = color,
            _ => {}
        }
    }
}

fn config_path(rel: &str) -> Option<std::path::PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|h| std::path::PathBuf::from(h).join(".config/veu").join(rel))
}

/// Parse `#RRGGBB` or `#RRGGBBAA` into an iced `Color`.
pub fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim().trim_start_matches('#');
    if s.len() < 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&s[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&s[4..6], 16).ok()? as f32 / 255.0;
    let a = if s.len() >= 8 {
        u8::from_str_radix(&s[6..8], 16).ok()? as f32 / 255.0
    } else {
        1.0
    };
    Some(Color { r, g, b, a })
}

fn hex(s: &str) -> Color {
    parse_color(s).expect("invalid hex in Theme::default()")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rrggbb() {
        let c = parse_color("#ff9500").unwrap();
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.584).abs() < 0.01);
        assert_eq!(c.b, 0.0);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn parse_rrggbbaa() {
        let c = parse_color("#1e1e2eee").unwrap();
        assert!((c.r - 0.118).abs() < 0.01);
        assert!((c.a - 0.933).abs() < 0.01);
    }

    #[test]
    fn parse_without_hash() {
        assert!(parse_color("ffffff").is_some());
    }

    #[test]
    fn parse_too_short_returns_none() {
        assert!(parse_color("#fff").is_none());
    }

    #[test]
    fn apply_key_updates_field() {
        let mut t = Theme::default();
        t.apply_key("accent", "#ff0000");
        assert!((t.accent.r - 1.0).abs() < 0.01);
        assert_eq!(t.accent.g, 0.0);
    }

    #[test]
    fn apply_unknown_key_is_noop() {
        let base = Theme::default();
        let mut t = Theme::default();
        t.apply_key("nonexistent", "#ff0000");
        assert_eq!(t, base);
    }

    #[test]
    fn parse_conf_overrides_only_given_keys() {
        let t = Theme::parse("accent = #ff0000\n# comment\n\ntext = #aabbcc\n");
        assert!((t.accent.r - 1.0).abs() < 0.01);
        assert_eq!(t.accent.g, 0.0);
        assert_eq!(t.background, Theme::default().background);
    }
}

use crossterm::style::Color;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorTheme {
    Default,
    Dark,
    Light,
    HighContrast,
}

impl FromStr for ColorTheme {
    type Err = std::convert::Infallible;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Ok(match name.to_ascii_lowercase().as_str() {
            "dark" => ColorTheme::Dark,
            "light" => ColorTheme::Light,
            "high_contrast" | "high-contrast" => ColorTheme::HighContrast,
            _ => ColorTheme::Default,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemePalette {
    pub header: Color,
    pub label: Color,
    pub ok: Color,
    pub warning: Color,
    pub critical: Color,
    pub accent: Color,
    pub muted: Color,
    pub selection: Color,
}

impl ThemePalette {
    fn default_palette() -> Self {
        ThemePalette {
            header: Color::Cyan,
            label: Color::White,
            ok: Color::Green,
            warning: Color::Yellow,
            critical: Color::Red,
            accent: Color::Magenta,
            muted: Color::DarkGrey,
            selection: Color::Black,
        }
    }

    fn dark_palette() -> Self {
        ThemePalette {
            header: Color::Cyan,
            label: Color::White,
            ok: Color::Green,
            warning: Color::Yellow,
            critical: Color::Red,
            accent: Color::Magenta,
            muted: Color::DarkGrey,
            selection: Color::Black,
        }
    }

    fn light_palette() -> Self {
        ThemePalette {
            header: Color::Blue,
            label: Color::Black,
            ok: Color::DarkGreen,
            warning: Color::DarkYellow,
            critical: Color::Red,
            accent: Color::DarkMagenta,
            muted: Color::DarkGrey,
            selection: Color::White,
        }
    }

    fn high_contrast_palette() -> Self {
        ThemePalette {
            header: Color::Yellow,
            label: Color::White,
            ok: Color::Green,
            warning: Color::Yellow,
            critical: Color::Red,
            accent: Color::Cyan,
            muted: Color::Grey,
            selection: Color::White,
        }
    }
}

pub fn palette_for(theme: &str) -> ThemePalette {
    match theme.parse().unwrap_or(ColorTheme::Default) {
        ColorTheme::Default => ThemePalette::default_palette(),
        ColorTheme::Dark => ThemePalette::dark_palette(),
        ColorTheme::Light => ThemePalette::light_palette(),
        ColorTheme::HighContrast => ThemePalette::high_contrast_palette(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_for_known_themes() {
        let default = palette_for("default");
        assert_eq!(default.header, Color::Cyan);
        assert_eq!(default.ok, Color::Green);

        let dark = palette_for("dark");
        assert_eq!(dark.header, Color::Cyan);

        let light = palette_for("light");
        assert_eq!(light.label, Color::Black);
        assert_eq!(light.header, Color::Blue);

        let high_contrast = palette_for("high_contrast");
        assert_eq!(high_contrast.header, Color::Yellow);
        assert_eq!(high_contrast.accent, Color::Cyan);
    }

    #[test]
    fn test_palette_for_unknown_theme_falls_back_to_default() {
        let unknown = palette_for("neon_punk");
        let default = palette_for("default");
        assert_eq!(unknown, default);
    }

    #[test]
    fn test_color_theme_deserialize() {
        let theme: ColorTheme = serde_json::from_str("\"dark\"").unwrap();
        assert_eq!(theme, ColorTheme::Dark);

        let theme: ColorTheme = serde_json::from_str("\"high_contrast\"").unwrap();
        assert_eq!(theme, ColorTheme::HighContrast);
    }

    #[test]
    fn test_color_theme_from_str_aliases() {
        assert_eq!(
            "HIGH_CONTRAST".parse::<ColorTheme>().unwrap(),
            ColorTheme::HighContrast
        );
        assert_eq!(
            "high-contrast".parse::<ColorTheme>().unwrap(),
            ColorTheme::HighContrast
        );
        assert_eq!(
            "unknown".parse::<ColorTheme>().unwrap(),
            ColorTheme::Default
        );
    }
}

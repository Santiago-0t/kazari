//! Colors and the built-in palettes.

use std::fmt;
use std::str::FromStr;

/// An opaque sRGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Builds a color from a `0xRRGGBB` literal, handy for constant tables.
    pub const fn hex(value: u32) -> Self {
        Self::rgb((value >> 16) as u8, (value >> 8) as u8, value as u8)
    }

    /// `#rrggbb`, the format most config files expect.
    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// ANSI SGR parameters for a 24-bit foreground color, e.g. `38;2;183;126;100`.
    pub fn to_ansi_fg(self) -> String {
        format!("38;2;{};{};{}", self.r, self.g, self.b)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("invalid hex color `{0}`; expected #rrggbb")]
pub struct ParseColorError(String);

impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let digits = s.strip_prefix('#').unwrap_or(s);
        if digits.len() != 6 {
            return Err(ParseColorError(s.to_owned()));
        }
        let value = u32::from_str_radix(digits, 16).map_err(|_| ParseColorError(s.to_owned()))?;
        Ok(Self::hex(value))
    }
}

/// A full system palette: UI roles plus the 16 ANSI terminal colors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    pub name: &'static str,
    pub background: Color,
    pub surface: Color,
    pub foreground: Color,
    pub muted: Color,
    pub accent: Color,
    /// black, red, green, yellow, blue, magenta, cyan, white, then the bright variants.
    pub ansi: [Color; 16],
}

impl fmt::Display for Palette {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

const fn ansi(normal: [u32; 8], bright: [u32; 8]) -> [Color; 16] {
    let mut out = [Color::rgb(0, 0, 0); 16];
    let mut i = 0;
    while i < 8 {
        out[i] = Color::hex(normal[i]);
        out[i + 8] = Color::hex(bright[i]);
        i += 1;
    }
    out
}

/// Palettes shipped with the app.
pub fn builtin() -> Vec<Palette> {
    vec![
        Palette {
            name: "Zenwritten",
            background: Color::hex(0x191919),
            surface: Color::hex(0x2a2a2a),
            foreground: Color::hex(0xbbbbbb),
            muted: Color::hex(0x8e8e8e),
            accent: Color::hex(0xb77e64),
            ansi: ansi(
                [
                    0x191919, 0xde6e7c, 0x819b69, 0xb77e64, 0x6099c0, 0xb279a7, 0x66a5ad, 0xbbbbbb,
                ],
                [
                    0x3d3839, 0xe8838f, 0x8bae68, 0xd68c67, 0x61abda, 0xcf86c1, 0x65b8c1, 0x8e8e8e,
                ],
            ),
        },
        Palette {
            name: "Catppuccin Mocha",
            background: Color::hex(0x1e1e2e),
            surface: Color::hex(0x313244),
            foreground: Color::hex(0xcdd6f4),
            muted: Color::hex(0x6c7086),
            accent: Color::hex(0xcba6f7),
            ansi: ansi(
                [
                    0x45475a, 0xf38ba8, 0xa6e3a1, 0xf9e2af, 0x89b4fa, 0xf5c2e7, 0x94e2d5, 0xbac2de,
                ],
                [
                    0x585b70, 0xf38ba8, 0xa6e3a1, 0xf9e2af, 0x89b4fa, 0xf5c2e7, 0x94e2d5, 0xa6adc8,
                ],
            ),
        },
        Palette {
            name: "Gruvbox Dark",
            background: Color::hex(0x282828),
            surface: Color::hex(0x3c3836),
            foreground: Color::hex(0xebdbb2),
            muted: Color::hex(0x928374),
            accent: Color::hex(0xfe8019),
            ansi: ansi(
                [
                    0x282828, 0xcc241d, 0x98971a, 0xd79921, 0x458588, 0xb16286, 0x689d6a, 0xa89984,
                ],
                [
                    0x928374, 0xfb4934, 0xb8bb26, 0xfabd2f, 0x83a598, 0xd3869b, 0x8ec07c, 0xebdbb2,
                ],
            ),
        },
        Palette {
            name: "Nord",
            background: Color::hex(0x2e3440),
            surface: Color::hex(0x3b4252),
            foreground: Color::hex(0xd8dee9),
            muted: Color::hex(0x4c566a),
            accent: Color::hex(0x88c0d0),
            ansi: ansi(
                [
                    0x3b4252, 0xbf616a, 0xa3be8c, 0xebcb8b, 0x81a1c1, 0xb48ead, 0x88c0d0, 0xe5e9f0,
                ],
                [
                    0x4c566a, 0xbf616a, 0xa3be8c, 0xebcb8b, 0x81a1c1, 0xb48ead, 0x8fbcbb, 0xeceff4,
                ],
            ),
        },
        Palette {
            name: "Tokyo Night",
            background: Color::hex(0x1a1b26),
            surface: Color::hex(0x24283b),
            foreground: Color::hex(0xc0caf5),
            muted: Color::hex(0x565f89),
            accent: Color::hex(0x7aa2f7),
            ansi: ansi(
                [
                    0x15161e, 0xf7768e, 0x9ece6a, 0xe0af68, 0x7aa2f7, 0xbb9af7, 0x7dcfff, 0xa9b1d6,
                ],
                [
                    0x414868, 0xf7768e, 0x9ece6a, 0xe0af68, 0x7aa2f7, 0xbb9af7, 0x7dcfff, 0xc0caf5,
                ],
            ),
        },
        Palette {
            name: "Rosé Pine",
            background: Color::hex(0x191724),
            surface: Color::hex(0x1f1d2e),
            foreground: Color::hex(0xe0def4),
            muted: Color::hex(0x6e6a86),
            accent: Color::hex(0xc4a7e7),
            ansi: ansi(
                [
                    0x26233a, 0xeb6f92, 0x31748f, 0xf6c177, 0x9ccfd8, 0xc4a7e7, 0xebbcba, 0xe0def4,
                ],
                [
                    0x6e6a86, 0xeb6f92, 0x31748f, 0xf6c177, 0x9ccfd8, 0xc4a7e7, 0xebbcba, 0xe0def4,
                ],
            ),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_prints_hex() {
        let color: Color = "#B77E64".parse().unwrap();
        assert_eq!(color, Color::rgb(183, 126, 100));
        assert_eq!(color.to_hex(), "#b77e64");
        assert_eq!("b77e64".parse::<Color>().unwrap(), color);
    }

    #[test]
    fn rejects_bad_hex() {
        assert!("#12345".parse::<Color>().is_err());
        assert!("#zzzzzz".parse::<Color>().is_err());
    }

    #[test]
    fn builtin_palettes_have_unique_names() {
        let palettes = builtin();
        let mut names: Vec<_> = palettes.iter().map(|p| p.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), palettes.len());
    }
}

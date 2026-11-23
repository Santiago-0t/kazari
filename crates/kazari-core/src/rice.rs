//! The user's choices: everything a rice is made of.

use std::fmt;

use crate::palette::Palette;

/// Nerd Fonts offered in the font picker. They must be installed on the system.
pub const FONTS: &[&str] = &[
    "JetBrainsMono Nerd Font",
    "CaskaydiaCove Nerd Font",
    "FiraCode Nerd Font",
    "Iosevka Nerd Font",
    "GeistMono Nerd Font",
];

/// Terminal emulators kazari knows how to theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terminal {
    WindowsTerminal,
    WezTerm,
    Alacritty,
}

impl Terminal {
    pub const ALL: [Terminal; 3] = [Terminal::WindowsTerminal, Terminal::WezTerm, Terminal::Alacritty];

    /// Whether the terminal can blur what is behind the window (acrylic).
    pub fn supports_acrylic(self) -> bool {
        matches!(self, Terminal::WindowsTerminal | Terminal::WezTerm)
    }
}

impl fmt::Display for Terminal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Terminal::WindowsTerminal => "Windows Terminal",
            Terminal::WezTerm => "WezTerm",
            Terminal::Alacritty => "Alacritty",
        })
    }
}

/// How terminal windows let the desktop show through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowEffect {
    Opaque,
    /// Plain transparency; `opacity` is a percentage (0-100).
    Transparent {
        opacity: u8,
    },
    /// Transparency plus a blur of what is behind the window.
    Acrylic {
        opacity: u8,
    },
}

impl WindowEffect {
    /// Opacity as a fraction in `0.0..=1.0`.
    pub fn opacity(self) -> f32 {
        match self {
            WindowEffect::Opaque => 1.0,
            WindowEffect::Transparent { opacity } | WindowEffect::Acrylic { opacity } => {
                f32::from(opacity.min(100)) / 100.0
            }
        }
    }

    /// The effect a terminal can actually render: acrylic degrades to transparency.
    pub fn for_terminal(self, terminal: Terminal) -> WindowEffect {
        match self {
            WindowEffect::Acrylic { opacity } if !terminal.supports_acrylic() => {
                WindowEffect::Transparent { opacity }
            }
            other => other,
        }
    }
}

/// Oh My Posh prompt layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptStyle {
    /// Path and git branch in plain colored text.
    Minimal,
    /// Powerline blocks with arrow separators.
    Powerline,
    /// Rounded "pill" segments.
    Pills,
}

impl PromptStyle {
    pub const ALL: [PromptStyle; 3] = [PromptStyle::Minimal, PromptStyle::Powerline, PromptStyle::Pills];
}

impl fmt::Display for PromptStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            PromptStyle::Minimal => "Minimal",
            PromptStyle::Powerline => "Powerline",
            PromptStyle::Pills => "Pills",
        })
    }
}

/// A complete rice, ready to be turned into config files.
#[derive(Debug, Clone, PartialEq)]
pub struct Rice {
    pub palette: Palette,
    pub font: String,
    pub font_size: u8,
    pub prompt: PromptStyle,
    pub terminal: Terminal,
    pub effect: WindowEffect,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acrylic_falls_back_to_transparency_on_alacritty() {
        let effect = WindowEffect::Acrylic { opacity: 80 };
        assert_eq!(
            effect.for_terminal(Terminal::Alacritty),
            WindowEffect::Transparent { opacity: 80 }
        );
        assert_eq!(effect.for_terminal(Terminal::WindowsTerminal), effect);
    }

    #[test]
    fn opacity_is_clamped() {
        assert_eq!(WindowEffect::Transparent { opacity: 150 }.opacity(), 1.0);
        assert_eq!(WindowEffect::Opaque.opacity(), 1.0);
        assert_eq!(WindowEffect::Acrylic { opacity: 85 }.opacity(), 0.85);
    }
}

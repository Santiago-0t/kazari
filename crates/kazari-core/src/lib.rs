//! kazari-core: everything kazari does, without a user interface.
//!
//! The flow is `Rice` (what the user picked) -> [`plan::plan`] (the files to write)
//! -> [`plan::write`] (back up and write them).

pub mod generators;
pub mod palette;
pub mod plan;
pub mod rice;
pub mod text;

pub use palette::{Color, Palette};
pub use rice::{PromptStyle, Rice, Terminal, WindowEffect};

#[cfg(test)]
pub(crate) mod test_support {
    use crate::palette;
    use crate::rice::{PromptStyle, Rice, Terminal, WindowEffect};

    /// Zenwritten, JetBrainsMono, minimal prompt, Windows Terminal, opaque.
    pub fn sample_rice() -> Rice {
        Rice {
            palette: palette::builtin().remove(0),
            font: "JetBrainsMono Nerd Font".into(),
            font_size: 12,
            prompt: PromptStyle::Minimal,
            terminal: Terminal::WindowsTerminal,
            effect: WindowEffect::Opaque,
        }
    }
}

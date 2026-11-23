//! YASB (status bar): CSS variables and font, injected at the top of `styles.css`.

use crate::rice::Rice;
use crate::text::upsert_block;

const START: &str = "/* kazari:start (generated, edits here are overwritten) */";
const END: &str = "/* kazari:end */";

/// The kazari block: palette variables the user's own rules can reference with `var(--kz-*)`.
pub fn block(rice: &Rice) -> String {
    let p = &rice.palette;
    format!(
        ":root {{\n  --kz-bg: {};\n  --kz-surface: {};\n  --kz-fg: {};\n  --kz-muted: {};\n  --kz-accent: {};\n  --kz-red: {};\n  --kz-green: {};\n  --kz-yellow: {};\n  --kz-blue: {};\n}}\n* {{ font-family: \"{}\"; }}\n.yasb-bar {{ background-color: var(--kz-bg); color: var(--kz-fg); }}",
        p.background,
        p.surface,
        p.foreground,
        p.muted,
        p.accent,
        p.ansi[1],
        p.ansi[2],
        p.ansi[3],
        p.ansi[4],
        rice.font,
    )
}

/// Updates an existing `styles.css`, leaving the user's rules untouched.
pub fn patch(existing: &str, rice: &Rice) -> String {
    upsert_block(existing, START, END, &block(rice))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::sample_rice;

    #[test]
    fn injects_variables_and_font() {
        let out = patch(".clock { color: var(--kz-accent); }", &sample_rice());
        assert!(out.contains("--kz-accent: #b77e64;"));
        assert!(out.contains("font-family: \"JetBrainsMono Nerd Font\""));
        assert!(out.contains(".clock { color: var(--kz-accent); }"));
    }
}

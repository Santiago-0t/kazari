//! Oh My Posh: a full prompt theme that uses the palette through `p:` references.

use serde_json::{Value, json};

use crate::rice::{PromptStyle, Rice};

/// Builds a complete theme file (`*.omp.json`).
pub fn theme(rice: &Rice) -> String {
    let p = &rice.palette;
    let palette = json!({
        "bg": p.background.to_hex(),
        "fg": p.foreground.to_hex(),
        "surface": p.surface.to_hex(),
        "accent": p.accent.to_hex(),
        "muted": p.muted.to_hex(),
        "git": p.ansi[2].to_hex(),
        "error": p.ansi[1].to_hex(),
    });

    let segments = match rice.prompt {
        PromptStyle::Minimal => minimal(),
        PromptStyle::Powerline => blocks("powerline", Some("\u{e0b0}")),
        PromptStyle::Pills => blocks("diamond", None),
    };

    let theme = json!({
        "$schema": "https://raw.githubusercontent.com/JanDeDobbeleer/oh-my-posh/main/themes/schema.json",
        "version": 3,
        "final_space": true,
        "palette": palette,
        "blocks": [
            { "type": "prompt", "alignment": "left", "segments": segments },
            { "type": "prompt", "alignment": "left", "newline": true, "segments": [
                { "type": "status", "style": "plain", "foreground": "p:accent",
                  "foreground_templates": ["{{ if gt .Code 0 }}p:error{{ end }}"],
                  "template": "\u{276f}", "properties": { "always_enabled": true } }
            ]}
        ]
    });
    serde_json::to_string_pretty(&theme).expect("static JSON") + "\n"
}

fn minimal() -> Value {
    json!([
        { "type": "path", "style": "plain", "foreground": "p:accent",
          "template": "{{ .Path }} ", "properties": { "style": "folder" } },
        { "type": "git", "style": "plain", "foreground": "p:muted",
          "template": "{{ .HEAD }} " }
    ])
}

fn blocks(style: &str, separator: Option<&str>) -> Value {
    let mut path = json!({ "type": "path", "style": style, "foreground": "p:bg", "background": "p:accent",
        "template": " {{ .Path }} ", "properties": { "style": "folder" } });
    let mut git = json!({ "type": "git", "style": style, "foreground": "p:bg", "background": "p:git",
        "template": " {{ .HEAD }} " });
    for segment in [&mut path, &mut git] {
        match separator {
            Some(symbol) => segment["powerline_symbol"] = json!(symbol),
            None => {
                segment["leading_diamond"] = json!("\u{e0b6}");
                segment["trailing_diamond"] = json!("\u{e0b4}");
            }
        }
    }
    json!([path, git])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::sample_rice;

    #[test]
    fn theme_exposes_the_palette() {
        let out: Value = serde_json::from_str(&theme(&sample_rice())).unwrap();
        assert_eq!(out["palette"]["accent"], "#b77e64");
        assert_eq!(out["blocks"][0]["segments"][0]["foreground"], "p:accent");
    }

    #[test]
    fn each_style_has_its_own_segment_shape() {
        let mut rice = sample_rice();
        rice.prompt = PromptStyle::Powerline;
        let out: Value = serde_json::from_str(&theme(&rice)).unwrap();
        assert_eq!(out["blocks"][0]["segments"][0]["style"], "powerline");

        rice.prompt = PromptStyle::Pills;
        let out: Value = serde_json::from_str(&theme(&rice)).unwrap();
        assert_eq!(out["blocks"][0]["segments"][0]["style"], "diamond");
    }
}

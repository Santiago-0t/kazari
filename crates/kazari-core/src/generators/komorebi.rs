//! komorebi (tiling window manager): window border colors.

use serde_json::{Value, json};

use crate::rice::Rice;
use crate::text::{EditError, object_at, parse_jsonc};

/// Updates an existing `komorebi.json`, keeping every setting kazari does not own.
pub fn patch(existing: &str, rice: &Rice) -> Result<String, EditError> {
    let mut config: Value = parse_jsonc(existing)?;
    let palette = &rice.palette;

    let root = config.as_object_mut().expect("parse_jsonc returns an object");
    root.insert("border".into(), json!(true));
    root.entry("border_width").or_insert(json!(4));

    let colours = object_at(root, "border_colours");
    colours.insert("single".into(), json!(palette.accent.to_hex()));
    colours.insert("stack".into(), json!(palette.ansi[2].to_hex()));
    colours.insert("monocle".into(), json!(palette.ansi[5].to_hex()));
    colours.insert("unfocused".into(), json!(palette.surface.to_hex()));

    Ok(serde_json::to_string_pretty(&config)? + "\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::sample_rice;

    #[test]
    fn sets_border_colours_and_keeps_user_settings() {
        let existing = r#"{ "default_workspace_padding": 10, "border_width": 8 }"#;
        let out: Value = serde_json::from_str(&patch(existing, &sample_rice()).unwrap()).unwrap();
        assert_eq!(out["default_workspace_padding"], 10);
        assert_eq!(out["border_width"], 8, "an existing width must be kept");
        assert_eq!(out["border"], true);
        assert_eq!(out["border_colours"]["single"], "#b77e64");
    }
}

//! Helpers to edit existing config files without clobbering the user's own settings.

use std::io::Read;

use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum EditError {
    #[error("could not parse JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("could not read JSON: {0}")]
    Io(#[from] std::io::Error),
    #[error("expected a JSON object at the top level")]
    NotAnObject,
}

/// Parses JSON that may contain `//` and `/* */` comments (JSONC), as Windows Terminal,
/// komorebi and fastfetch allow. An empty input yields an empty object; any other
/// top-level value than an object is rejected, so callers can rely on getting a map.
pub fn parse_jsonc(input: &str) -> Result<Value, EditError> {
    if input.trim().is_empty() {
        return Ok(Value::Object(Default::default()));
    }
    let mut stripped = String::new();
    json_comments::StripComments::new(input.as_bytes()).read_to_string(&mut stripped)?;
    let value: Value = serde_json::from_str(&stripped)?;
    if value.is_object() {
        Ok(value)
    } else {
        Err(EditError::NotAnObject)
    }
}

/// Returns the object stored at `key` inside `map`, creating it (or replacing a non-object).
pub fn object_at<'a>(
    map: &'a mut serde_json::Map<String, Value>,
    key: &str,
) -> &'a mut serde_json::Map<String, Value> {
    let entry = map
        .entry(key)
        .or_insert_with(|| Value::Object(Default::default()));
    if !entry.is_object() {
        *entry = Value::Object(Default::default());
    }
    entry.as_object_mut().expect("just ensured an object")
}

/// Replaces the text between `start` and `end` marker lines, or appends a new marked block.
/// Running it twice with the same block gives the same result.
pub fn upsert_block(existing: &str, start: &str, end: &str, block: &str) -> String {
    let marked = format!("{start}\n{}\n{end}", block.trim_end());
    if let (Some(from), Some(to)) = (existing.find(start), existing.find(end))
        && from < to
    {
        let after = to + end.len();
        return format!("{}{marked}{}", &existing[..from], &existing[after..]);
    }
    if existing.trim().is_empty() {
        format!("{marked}\n")
    } else {
        format!("{marked}\n\n{existing}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_with_comments() {
        let value = parse_jsonc("{\n  // a comment\n  \"a\": 1 /* inline */\n}").unwrap();
        assert_eq!(value["a"], 1);
    }

    #[test]
    fn rejects_non_object_roots() {
        assert!(matches!(parse_jsonc("[1, 2]"), Err(EditError::NotAnObject)));
    }

    #[test]
    fn empty_input_is_an_empty_object() {
        assert_eq!(parse_jsonc("  \n").unwrap(), serde_json::json!({}));
    }

    #[test]
    fn upsert_block_is_idempotent_and_keeps_user_text() {
        let user = ".bar { color: red; }\n";
        let once = upsert_block(user, "/* s */", "/* e */", ":root { --a: 1; }");
        let twice = upsert_block(&once, "/* s */", "/* e */", ":root { --a: 1; }");
        assert_eq!(once, twice);
        assert!(once.ends_with(user));

        let changed = upsert_block(&once, "/* s */", "/* e */", ":root { --a: 2; }");
        assert!(changed.contains("--a: 2") && !changed.contains("--a: 1"));
    }
}

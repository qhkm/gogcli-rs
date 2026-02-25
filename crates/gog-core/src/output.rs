// gog-core output module
// Ported from internal/outfmt/outfmt.go

use std::io::Write;

use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error type (local, avoids depending on error.rs stub)
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum OutputError {
    #[error("invalid output mode: cannot combine --json and --plain")]
    ConflictingFlags,

    #[error("serialize value: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("write output: {0}")]
    Io(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// OutputMode
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputMode {
    #[default]
    Text,
    Json,
    Plain, // TSV
}

// ---------------------------------------------------------------------------
// OutputConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct OutputConfig {
    pub mode: OutputMode,
    pub results_only: bool,
    pub select_fields: Vec<String>,
}

impl OutputConfig {
    /// Build an `OutputConfig` from CLI flag values.
    /// Returns an error if both `json` and `plain` are true.
    pub fn from_flags(json: bool, plain: bool) -> Result<Self, String> {
        if json && plain {
            return Err("invalid output mode (cannot combine --json and --plain)".to_string());
        }
        let mode = if json {
            OutputMode::Json
        } else if plain {
            OutputMode::Plain
        } else {
            OutputMode::Text
        };
        Ok(OutputConfig {
            mode,
            results_only: false,
            select_fields: Vec::new(),
        })
    }

    pub fn is_json(&self) -> bool {
        self.mode == OutputMode::Json
    }

    pub fn is_plain(&self) -> bool {
        self.mode == OutputMode::Plain
    }
}

// ---------------------------------------------------------------------------
// write_json
// ---------------------------------------------------------------------------

/// Serialize `value` to pretty-printed JSON, applying any configured transforms.
pub fn write_json<W: Write>(
    w: &mut W,
    value: &impl Serialize,
    config: &OutputConfig,
) -> Result<(), OutputError> {
    // Roundtrip through serde_json::Value so we can apply transforms.
    let mut v: Value = serde_json::to_value(value)?;

    if config.results_only {
        v = unwrap_primary(v);
    }

    if !config.select_fields.is_empty() {
        v = select_fields(v, &config.select_fields);
    }

    // Pretty-print without HTML escaping (mirrors Go's SetEscapeHTML(false)).
    let s = serde_json::to_string_pretty(&v)?;
    w.write_all(s.as_bytes())?;
    // Go's json.Encoder adds a trailing newline; keep parity.
    w.write_all(b"\n")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// unwrap_primary  (--results-only logic)
// ---------------------------------------------------------------------------

/// Unwrap the "primary" payload from an envelope object.
///
/// Precedence:
/// 1. Explicit `"results"` key.
/// 2. Single non-meta key.
/// 3. Any array-typed candidate.
/// 4. Known result-list keys.
/// 5. Fall back to `v` unchanged.
fn unwrap_primary(v: Value) -> Value {
    let m = match v {
        Value::Object(ref map) => map.clone(),
        other => return other,
    };

    // 1. Explicit "results" key.
    if let Some(results) = m.get("results") {
        return results.clone();
    }

    // Meta / envelope keys to skip.
    const META: &[&str] = &[
        "nextPageToken",
        "next_cursor",
        "has_more",
        "count",
        "query",
        "dry_run",
        "dryRun",
        "op",
        "action",
        "note",
        "notes",
    ];

    let candidates: Vec<&str> = m
        .keys()
        .filter(|k| !META.contains(&k.as_str()))
        .map(|k| k.as_str())
        .collect();

    // 2. Single non-meta key.
    if candidates.len() == 1 {
        return m[candidates[0]].clone();
    }

    // 3. Any array candidate — prefer the first one found.
    for k in &candidates {
        if m[*k].is_array() {
            return m[*k].clone();
        }
    }

    // 4. Known result-list keys (in priority order).
    const KNOWN: &[&str] = &[
        "files",
        "threads",
        "messages",
        "labels",
        "events",
        "calendars",
        "courses",
        "topics",
        "announcements",
        "materials",
        "coursework",
        "submissions",
        "invitations",
        "guardians",
        "notes",
        "contacts",
        "people",
        "tasks",
        "lists",
        "groups",
        "members",
        "drives",
        "rules",
        "colors",
        "spaces",
        "request",
    ];
    for k in KNOWN {
        if let Some(val) = m.get(*k) {
            return val.clone();
        }
    }

    // 5. Fall through.
    v
}

// ---------------------------------------------------------------------------
// select_fields  (--select logic)
// ---------------------------------------------------------------------------

/// Project `v` to only the specified fields.
/// When `v` is an array, each element is projected.
fn select_fields(v: Value, fields: &[String]) -> Value {
    match v {
        Value::Array(arr) => {
            let projected = arr
                .into_iter()
                .map(|item| select_fields_from_item(item, fields))
                .collect();
            Value::Array(projected)
        }
        other => select_fields_from_item(other, fields),
    }
}

fn select_fields_from_item(v: Value, fields: &[String]) -> Value {
    let m = match v {
        Value::Object(map) => map,
        other => return other,
    };

    let mut out = serde_json::Map::new();
    for f in fields {
        // Build a temporary Value::Object to allow get_at_path to work.
        let tmp = Value::Object(m.clone());
        if let Some(val) = get_at_path(&tmp, f) {
            out.insert(f.clone(), val);
        }
    }
    Value::Object(out)
}

// ---------------------------------------------------------------------------
// get_at_path  (dot-path access)
// ---------------------------------------------------------------------------

/// Traverse `v` along a dot-separated `path` and return the value at that
/// location, or `None` if any segment is missing.
fn get_at_path(v: &Value, path: &str) -> Option<Value> {
    let path = path.trim();
    if path.is_empty() {
        return None;
    }

    let mut cur = v;
    // We need owned intermediate values for the loop; use a local holder.
    let mut _owned: Value;

    let segs: Vec<&str> = path.split('.').collect();
    let last = segs.len() - 1;

    for (i, seg) in segs.iter().enumerate() {
        let seg = seg.trim();
        if seg.is_empty() {
            return None;
        }

        match cur {
            Value::Object(map) => {
                let next = map.get(seg)?;
                if i == last {
                    return Some(next.clone());
                }
                _owned = next.clone();
                cur = &_owned;
            }
            Value::Array(arr) => {
                let idx: usize = seg.parse().ok()?;
                let next = arr.get(idx)?;
                if i == last {
                    return Some(next.clone());
                }
                _owned = next.clone();
                cur = &_owned;
            }
            _ => return None,
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ------------------------------------------------------------------
    // OutputConfig::from_flags
    // ------------------------------------------------------------------

    #[test]
    fn test_output_config_from_flags_json() {
        let cfg = OutputConfig::from_flags(true, false).unwrap();
        assert_eq!(cfg.mode, OutputMode::Json);
        assert!(cfg.is_json());
        assert!(!cfg.is_plain());
    }

    #[test]
    fn test_output_config_from_flags_plain() {
        let cfg = OutputConfig::from_flags(false, true).unwrap();
        assert_eq!(cfg.mode, OutputMode::Plain);
        assert!(!cfg.is_json());
        assert!(cfg.is_plain());
    }

    #[test]
    fn test_output_config_from_flags_default() {
        let cfg = OutputConfig::from_flags(false, false).unwrap();
        assert_eq!(cfg.mode, OutputMode::Text);
        assert!(!cfg.is_json());
        assert!(!cfg.is_plain());
    }

    #[test]
    fn test_output_config_from_flags_both_error() {
        let err = OutputConfig::from_flags(true, true).unwrap_err();
        assert!(err.contains("cannot combine"));
    }

    // ------------------------------------------------------------------
    // unwrap_primary
    // ------------------------------------------------------------------

    #[test]
    fn test_unwrap_primary_results_key() {
        let v = json!({
            "results": [1, 2, 3],
            "nextPageToken": "abc"
        });
        let out = unwrap_primary(v);
        assert_eq!(out, json!([1, 2, 3]));
    }

    #[test]
    fn test_unwrap_primary_single_candidate() {
        let v = json!({
            "messages": [{"id": "1"}, {"id": "2"}],
            "nextPageToken": "tok"
        });
        let out = unwrap_primary(v);
        assert_eq!(out, json!([{"id": "1"}, {"id": "2"}]));
    }

    #[test]
    fn test_unwrap_primary_array_preference() {
        // "label" is a string candidate, "items" is an array candidate.
        // Array should be preferred.
        let v = json!({
            "label": "hello",
            "items": [1, 2, 3]
        });
        let out = unwrap_primary(v);
        assert_eq!(out, json!([1, 2, 3]));
    }

    #[test]
    fn test_unwrap_primary_passthrough() {
        // Non-object values are returned as-is.
        let v = json!([10, 20, 30]);
        let out = unwrap_primary(v.clone());
        assert_eq!(out, v);

        let v2 = json!("just a string");
        let out2 = unwrap_primary(v2.clone());
        assert_eq!(out2, v2);
    }

    // ------------------------------------------------------------------
    // select_fields
    // ------------------------------------------------------------------

    #[test]
    fn test_select_fields_flat() {
        let v = json!({"id": "1", "name": "Alice", "email": "alice@example.com"});
        let fields = vec!["id".to_string(), "name".to_string()];
        let out = select_fields(v, &fields);
        assert_eq!(out, json!({"id": "1", "name": "Alice"}));
    }

    #[test]
    fn test_select_fields_array() {
        let v = json!([
            {"id": "1", "name": "Alice", "email": "a@b.com"},
            {"id": "2", "name": "Bob",   "email": "b@b.com"}
        ]);
        let fields = vec!["id".to_string(), "name".to_string()];
        let out = select_fields(v, &fields);
        assert_eq!(
            out,
            json!([
                {"id": "1", "name": "Alice"},
                {"id": "2", "name": "Bob"}
            ])
        );
    }

    // ------------------------------------------------------------------
    // get_at_path
    // ------------------------------------------------------------------

    #[test]
    fn test_get_at_path_nested() {
        let v = json!({"user": {"name": "Alice"}});
        let result = get_at_path(&v, "user.name");
        assert_eq!(result, Some(json!("Alice")));
    }

    #[test]
    fn test_get_at_path_missing() {
        let v = json!({"user": {"name": "Alice"}});
        let result = get_at_path(&v, "user.email");
        assert_eq!(result, None);
    }

    // ------------------------------------------------------------------
    // write_json
    // ------------------------------------------------------------------

    #[test]
    fn test_write_json_basic() {
        let cfg = OutputConfig::default();
        let value = json!({"hello": "world"});
        let mut buf = Vec::new();
        write_json(&mut buf, &value, &cfg).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // Should be pretty-printed JSON ending with a newline.
        assert!(output.contains("\"hello\""));
        assert!(output.contains("\"world\""));
        assert!(output.ends_with('\n'));
        // Must be valid JSON.
        let _: Value = serde_json::from_str(output.trim()).unwrap();
    }

    #[test]
    fn test_write_json_results_only() {
        let cfg = OutputConfig {
            mode: OutputMode::Json,
            results_only: true,
            select_fields: Vec::new(),
        };
        let value = json!({
            "results": [{"id": "1"}, {"id": "2"}],
            "nextPageToken": "token"
        });
        let mut buf = Vec::new();
        write_json(&mut buf, &value, &cfg).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let parsed: Value = serde_json::from_str(output.trim()).unwrap();
        // Should have been unwrapped to the array.
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 2);
    }
}

//! Unit Process Communication Protocol
//!
//! COGTOME units communicate with the runtime via stdout using a strict protocol.
//!
//! # Protocol v1 (NDJSON with type tags)
//!
//! Each line of stdout is a JSON object with a required `type` field:
//!
//! ```json
//! {"type": "result", "data": {...}}
//! {"type": "log", "level": "info", "message": "..."}
//! {"type": "progress", "percent": 50}
//! ```
//!
//! Only the line with `{"type": "result", ...}` is used as the unit's output.
//! All other lines are silently ignored.
//!
//! # Protocol v2 (simple — preferred)
//!
//! Units should output exactly ONE line of valid JSON to stdout — the result.
//! Nothing else should go to stdout (logs go to stderr).
//!
//! # Legacy / permissive mode
//!
//! If the first line is not `{"type": "result", ...}`, we scan all lines looking
//! for one with `type: "result"`. If found, that line is the result. This allows
//! gradual migration without breaking existing units.
//!
//! # Error protocol
//!
//! Units exit with structured exit codes:
//! - `0`  → success
//! - `1`  → input error (invalid input passed by caller)
//! - `2`  → retryable transient failure
//! - `3`  → dependency unavailable
//! - `>3` → non-zero: unspecified fatal error
//!
//! Stderr is always treated as unstructured log/error output (not part of protocol).

use serde::Serialize;
use serde_json::Value;

// ProtocolLine intentionally removed — kept type info in docstring only.
// If needed in future, uncomment:
// pub enum ProtocolLine { Result(Value), Log { level, message }, Raw(String) }

/// NDJSON output parser with type-tag awareness.
///
/// Scans all lines, extracts exactly one `{"type":"result",...}` line,
/// and silently drops everything else.
pub fn parse_ndjson_output(stdout: &str) -> Result<Value, ParseError> {
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.is_empty() {
        return Err(ParseError::empty_output());
    }

    let mut results: Vec<Value> = Vec::new();
    let mut raw_lines: Vec<&str> = Vec::new();

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Try to parse as JSON
        match serde_json::from_str::<Value>(trimmed) {
            Ok(value) => {
                // Check for type tag
                if let Some(obj) = value.as_object() {
                    if let Some(type_val) = obj.get("type") {
                        match type_val.as_str() {
                            Some("result") => {
                                results.push(value);
                            }
                            // log / progress — ignore
                            _ => {}
                        }
                    } else {
                        // JSON but no type field — treat as raw, collect for legacy compat
                        raw_lines.push(trimmed);
                    }
                } else {
                    // Primitive JSON (number, string, etc.) — treat as raw
                    raw_lines.push(trimmed);
                }
            }
            Err(_) => {
                // Not valid JSON — treat as raw log line
                raw_lines.push(trimmed);
            }
        }
    }

    // Prefer typed result, fall back to legacy heuristic
    if !results.is_empty() {
        if results.len() > 1 {
            eprintln!("[protocol] WARNING: multiple result lines in NDJSON output, using first");
        }
        return Ok(results.remove(0));
    }

    // Legacy compat: if no typed result found, look for first line that parses as JSON
    // (many existing units just print a bare JSON object)
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
            // Found at least one valid JSON line — use it
            return Ok(value);
        }
    }

    // Nothing parseable found
    let sample = lines.first().map(|l| &l[..l.len().min(100)]).unwrap_or("");
    Err(ParseError::from_no_result(&sample))
}

/// Validate that the unit output is a valid JSON value.
/// Returns the value on success.
#[allow(dead_code)]
pub fn parse_single_line_output(stdout: &str) -> Result<Value, ParseError> {
    let first_line = stdout.lines().next().unwrap_or("").trim();

    if first_line.is_empty() {
        return Err(ParseError::empty_output());
    }

    // Check for NDJSON result line
    if let Ok(value) = serde_json::from_str::<Value>(first_line) {
        if let Some(obj) = value.as_object() {
            if let Some(type_val) = obj.get("type") {
                if type_val.as_str() == Some("result") {
                    return Ok(value);
                }
            }
        }
        // Bare JSON without type field — use it (legacy compat)
        return Ok(value);
    }

    // First line is not JSON — error
    Err(ParseError::from_not_json(&first_line))
}

/// Errors that can occur when parsing unit output.
#[derive(Debug, Clone, Serialize)]
pub struct ParseError {
    pub kind: String,
    pub detail: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "protocol error [{}]: {}", self.kind, self.detail)
    }
}

impl ParseError {
    pub fn empty_output() -> Self {
        Self {
            kind: "empty_output".to_string(),
            detail: "unit produced no stdout output".to_string(),
        }
    }

    pub fn from_no_result(sample: &str) -> Self {
        Self {
            kind: "no_result_found".to_string(),
            detail: format!(
                "no result line found in NDJSON output; first non-JSON: {}",
                sample
            ),
        }
    }

    #[allow(dead_code)]
    pub fn from_not_json(sample: &str) -> Self {
        Self {
            kind: "not_json".to_string(),
            detail: format!(
                "stdout first line is not valid JSON: {}",
                sample
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ndjson_result() {
        let output = r#"{"type":"log","level":"info","message":"starting fetch"}
{"type":"result","data":{"url":"https://example.com","content":"hello"}}
{"type":"progress","percent":100}"#;
        let result = parse_ndjson_output(output).unwrap();
        assert_eq!(result["data"]["url"], "https://example.com");
    }

    #[test]
    fn test_parse_single_json_line() {
        let output = r#"{"url":"https://example.com","content":"hello"}"#;
        let result = parse_ndjson_output(output).unwrap();
        assert_eq!(result["url"], "https://example.com");
    }

    #[test]
    fn test_parse_empty() {
        let err = parse_ndjson_output("").unwrap_err();
        assert_eq!(err.kind, "empty_output");
    }

    #[test]
    fn test_parse_no_result() {
        let output = r#"some raw log line
another log"#;
        let err = parse_ndjson_output(output).unwrap_err();
        assert_eq!(err.kind, "no_result_found");
    }

    #[test]
    fn test_parse_first_line_only() {
        // Simple single-line output (expected common case)
        let output = r#"{"status":"ok","data":[1,2,3]}"#;
        let result = parse_ndjson_output(output).unwrap();
        assert_eq!(result["status"], "ok");
    }

    #[test]
    fn test_ndjson_with_extra_whitespace() {
        let output = "   \n  {\"type\":\"result\",\"data\":42}  \n  \n";
        let result = parse_ndjson_output(output).unwrap();
        assert_eq!(result["data"], 42);
    }
}

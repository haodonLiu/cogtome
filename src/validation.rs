use anyhow::Result;
use serde_json::Value;

/// Validates input against a JSON Schema.
/// Returns Ok(()) on success, or Err with detailed validation error.
pub fn validate_input(input: &Value, schema: &Value) -> Result<()> {
    match jsonschema::validate(schema, input) {
        Ok(()) => Ok(()),
        Err(validation_error) => {
            anyhow::bail!("input validation failed: {}", validation_error)
        }
    }
}

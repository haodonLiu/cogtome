//! Structured error types for COGTOME runtime.
//!
//! Each error carries a layer, code, message, hint, and retryability flag
//! to enable precise error handling at every level of the execution stack.

use serde::Serialize;
use std::fmt;

/// Error layers: which component generated the error.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorLayer {
    Runtime,
    Motif,
    Unit,
    Validation,
}

impl fmt::Display for ErrorLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorLayer::Runtime => write!(f, "runtime"),
            ErrorLayer::Motif => write!(f, "motif"),
            ErrorLayer::Unit => write!(f, "unit"),
            ErrorLayer::Validation => write!(f, "validation"),
        }
    }
}

/// Error codes for machine-readable error categorization.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// E_RUNTIME: internal runtime error
    ERuntime,
    /// E_MOTIF_PARSE: failed to parse motif manifest
    EMotifParse,
    /// E_MOTIF_EXEC: motif execution failed
    EMotifExec,
    /// E_UNIT_NOT_FOUND: requested unit does not exist
    EUnitNotFound,
    /// E_UNIT_EXEC: unit process execution failed
    EUnitExec,
    /// E_UNIT_TIMEOUT: unit exceeded its timeout
    EUnitTimeout,
    /// E_UNIT_RETRYABLE: unit exited with code 2 (retryable)
    EUnitRetryable,
    /// E_UNIT_DEP_UNAVAILABLE: unit exited with code 3 (dependency unavailable)
    EUnitDepUnavailable,
    /// E_UNIT_INPUT_ERROR: unit exited with code 1 (invalid input)
    EUnitInputError,
    /// E_UNIT_NONZERO: unit exited with non-zero code (not 0/1/2/3)
    EUnitNonzero,
    /// E_VALIDATION: input or manifest validation failed
    EValidation,
    /// E_STRUCTURE_PARSE: failed to parse structure manifest
    EStructureParse,
    /// E_COMPLEX_NOT_FOUND: requested complex does not exist
    EComplexNotFound,
    /// E_STRUCTURE_NOT_FOUND: requested structure does not exist
    EStructureNotFound,
    /// E_MOTIF_NOT_FOUND: requested motif does not exist
    EMotifNotFound,
    /// E_FOREACH_LIMIT: foreach iteration limit exceeded
    EForeachLimit,
    /// E_MAX_ITERATIONS_HARD: absolute max_iterations_hard limit exceeded
    EMaxIterationsHard,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::ERuntime => write!(f, "E_RUNTIME"),
            ErrorCode::EMotifParse => write!(f, "E_MOTIF_PARSE"),
            ErrorCode::EMotifExec => write!(f, "E_MOTIF_EXEC"),
            ErrorCode::EUnitNotFound => write!(f, "E_UNIT_NOT_FOUND"),
            ErrorCode::EUnitExec => write!(f, "E_UNIT_EXEC"),
            ErrorCode::EUnitTimeout => write!(f, "E_UNIT_TIMEOUT"),
            ErrorCode::EUnitRetryable => write!(f, "E_UNIT_RETRYABLE"),
            ErrorCode::EUnitDepUnavailable => write!(f, "E_UNIT_DEP_UNAVAILABLE"),
            ErrorCode::EUnitInputError => write!(f, "E_UNIT_INPUT_ERROR"),
            ErrorCode::EUnitNonzero => write!(f, "E_UNIT_NONZERO"),
            ErrorCode::EValidation => write!(f, "E_VALIDATION"),
            ErrorCode::EStructureParse => write!(f, "E_STRUCTURE_PARSE"),
            ErrorCode::EComplexNotFound => write!(f, "E_COMPLEX_NOT_FOUND"),
            ErrorCode::EStructureNotFound => write!(f, "E_STRUCTURE_NOT_FOUND"),
            ErrorCode::EMotifNotFound => write!(f, "E_MOTIF_NOT_FOUND"),
            ErrorCode::EForeachLimit => write!(f, "E_FOREACH_LIMIT"),
            ErrorCode::EMaxIterationsHard => write!(f, "E_MAX_ITERATIONS_HARD"),
        }
    }
}

/// Unit exit code semantics.
#[derive(Debug, Clone, Copy)]
pub enum ExitCode {
    Success = 0,
    /// Exit code 1: input error — do not retry
    InputError = 1,
    /// Exit code 2: retryable error — may be retried
    Retryable = 2,
    /// Exit code 3: dependency unavailable — do not retry
    DepUnavailable = 3,
}

impl From<i32> for ExitCode {
    fn from(code: i32) -> Self {
        match code {
            0 => ExitCode::Success,
            1 => ExitCode::InputError,
            2 => ExitCode::Retryable,
            3 => ExitCode::DepUnavailable,
            _ => ExitCode::Success, // treat unknown as success for now
        }
    }
}

/// Structured error type for COGTOME runtime.
#[derive(Debug, Clone, Serialize)]
pub struct CogtomeError {
    pub layer: ErrorLayer,
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    pub retryable: bool,
}

impl CogtomeError {
    pub fn new(layer: ErrorLayer, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            layer,
            code,
            message: message.into(),
            hint: None,
            retryable: false,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn retryable(mut self) -> Self {
        self.retryable = true;
        self
    }

    pub fn layer_runtime() -> Self {
        Self::new(ErrorLayer::Runtime, ErrorCode::ERuntime, "Runtime error")
    }

    pub fn layer_unit() -> Self {
        Self::new(ErrorLayer::Unit, ErrorCode::EUnitExec, "Unit execution error")
    }

    pub fn layer_motif() -> Self {
        Self::new(ErrorLayer::Motif, ErrorCode::EMotifExec, "Motif execution error")
    }

    pub fn layer_validation() -> Self {
        Self::new(ErrorLayer::Validation, ErrorCode::EValidation, "Validation error")
    }

    /// Convert from an exit code + unit name + stderr to a CogtomeError.
    pub fn from_exit_code(code: i32, unit_name: &str, stderr: &str) -> Self {
        match code {
            1 => Self::new(
                ErrorLayer::Unit,
                ErrorCode::EUnitInputError,
                format!("Unit '{}' exited with input error (code 1): {}", unit_name, stderr),
            )
            .with_hint("Check the input passed to the unit; it may not match the unit's expected schema"),
            2 => Self::new(
                ErrorLayer::Unit,
                ErrorCode::EUnitRetryable,
                format!("Unit '{}' exited with retryable error (code 2): {}", unit_name, stderr),
            )
            .retryable()
            .with_hint("Unit signaled a transient failure; retrying may resolve it"),
            3 => Self::new(
                ErrorLayer::Unit,
                ErrorCode::EUnitDepUnavailable,
                format!("Unit '{}' exited with dependency unavailable (code 3): {}", unit_name, stderr),
            )
            .with_hint("The unit's external dependency is unavailable; check the service is running"),
            _ => Self::new(
                ErrorLayer::Unit,
                ErrorCode::EUnitNonzero,
                format!("Unit '{}' exited with code {}: {}", unit_name, code, stderr),
            )
            .with_hint("Unit exited abnormally; check unit logs for details"),
        }
    }
}

impl fmt::Display for CogtomeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.layer, self.code, self.message)?;
        if let Some(ref hint) = self.hint {
            write!(f, " (hint: {})", hint)?;
        }
        Ok(())
    }
}

impl std::error::Error for CogtomeError {}

/// Helper to convert anyhow::Error to CogtomeError when crossing module boundaries.
/// For errors already containing a CogtomeError, returns it as-is.
impl From<anyhow::Error> for CogtomeError {
    fn from(e: anyhow::Error) -> Self {
        // Check if the anyhow error wraps a CogtomeError
        for cause in e.chain() {
            if let Some(cog_err) = cause.downcast_ref::<CogtomeError>() {
                return cog_err.clone();
            }
        }
        // Fallback: wrap in a generic runtime error
        Self::layer_runtime()
            .with_hint(format!("Internal error: {}", e))
    }
}

// CogtomeError implements std::error::Error, so anyhow provides the From impl automatically.
// Use `err.into()` or `anyhow::anyhow!(err)` to convert.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cogtome_error_display() {
        let err = CogtomeError::new(
            ErrorLayer::Unit,
            ErrorCode::EUnitTimeout,
            "Unit 'foo' timed out after 30s",
        )
        .with_hint("Increase timeout_secs in config");

        let display = format!("{}", err);
        assert!(display.contains("E_UNIT_TIMEOUT"));
        assert!(display.contains("hint"));
        assert!(display.contains("30s"));
    }

    #[test]
    fn test_from_exit_code() {
        let e1 = CogtomeError::from_exit_code(1, "my-unit", "bad input");
        assert!(matches!(e1.code, ErrorCode::EUnitInputError));
        assert!(!e1.retryable);

        let e2 = CogtomeError::from_exit_code(2, "my-unit", "network timeout");
        assert!(matches!(e2.code, ErrorCode::EUnitRetryable));
        assert!(e2.retryable);

        let e3 = CogtomeError::from_exit_code(3, "my-unit", "db down");
        assert!(matches!(e3.code, ErrorCode::EUnitDepUnavailable));
        assert!(!e3.retryable);
    }

    #[test]
    fn test_serialization() {
        let err = CogtomeError::new(ErrorLayer::Motif, ErrorCode::EMotifParse, "bad yaml")
            .with_hint("check syntax");
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"layer\":\"motif\""));
        assert!(json.contains("\"code\":\"e_motif_parse\""));
        assert!(json.contains("hint"));
    }
}

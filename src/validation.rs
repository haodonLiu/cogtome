use crate::discovery::SkillsDir;
use crate::engine::motif_manifest::{
    AggregateBlock, AggregateMode, FlowStep, ForeachBlock, MotifManifest, RetryConfig,
    StepErrorStrategy, StructureManifest,
};
use crate::error::{CogtomeError, ErrorCode, ErrorLayer};
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::Path;

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

// ============================================================================
// Motif Manifest Validation
// ============================================================================

/// Validation error with detailed context
#[derive(Debug)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(path: &str, message: &str) -> Self {
        Self {
            path: path.to_string(),
            message: message.to_string(),
        }
    }

    pub fn into_error(self) -> CogtomeError {
        CogtomeError::new(
            ErrorLayer::Validation,
            ErrorCode::EValidation,
            format!("[{}] {}", self.path, self.message),
        )
    }
}

/// Validates a MotifManifest and returns a list of validation errors.
/// Empty list means validation passed.
pub fn validate_motif(motif: &MotifManifest) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Validate flow has at least one step
    if motif.flow.is_empty() {
        errors.push(ValidationError::new(
            "flow",
            "motif flow cannot be empty",
        ));
    }

    // Validate each flow step
    for (idx, step) in motif.flow.iter().enumerate() {
        let step_path = format!("flow[{}]", idx);

        // Validate step has either unit or foreach (not both, not neither)
        match (&step.unit, &step.foreach) {
            (None, None) => {
                errors.push(ValidationError::new(
                    &format!("{}.name", step_path),
                    &format!("FlowStep '{}' must have either 'unit' or 'foreach'", step.name),
                ));
            }
            (Some(_), Some(_)) => {
                errors.push(ValidationError::new(
                    &format!("{}.name", step_path),
                    &format!("FlowStep '{}' has both 'unit' and 'foreach' - they are mutually exclusive", step.name),
                ));
            }
            _ => {}
        }

        // Validate retry config if present
        if let Some(ref retry) = step.retry {
            if let Some(err) = validate_retry(retry, &step_path) {
                errors.push(err);
            }
        }

        // Validate on_error + fallback consistency
        if step.on_error == Some(StepErrorStrategy::Fallback) && step.fallback.is_none() {
            errors.push(ValidationError::new(
                &format!("{}.on_error", step_path),
                &format!("FlowStep '{}' has on_error=fallback but no fallback value", step.name),
            ));
        }

        // Validate foreach block if present
        if let Some(ref foreach) = step.foreach {
            if let Some(err) = validate_foreach(foreach, &step_path) {
                errors.push(err);
            }
        }
    }

    errors
}

fn validate_retry(retry: &RetryConfig, step_path: &str) -> Option<ValidationError> {
    if retry.max < 1 {
        return Some(ValidationError::new(
            &format!("{}.retry.max", step_path),
            &format!("retry.max must be >= 1, got {}", retry.max),
        ));
    }
    None
}

fn validate_foreach(foreach: &ForeachBlock, step_path: &str) -> Option<ValidationError> {
    let foreach_path = format!("{}.foreach", step_path);

    // Validate max_iterations
    if foreach.max_iterations < 1 {
        return Some(ValidationError::new(
            &format!("{}.max_iterations", foreach_path),
            &format!("max_iterations must be >= 1, got {}", foreach.max_iterations),
        ));
    }

    // Validate aggregate mode completeness
    if let Some(err) = validate_aggregate(&foreach.aggregate, &foreach_path) {
        return Some(err);
    }

    // Validate foreach flow is not empty
    if foreach.flow.is_empty() {
        return Some(ValidationError::new(
            &format!("{}.flow", foreach_path),
            "foreach flow cannot be empty",
        ));
    }

    None
}

fn validate_aggregate(aggregate: &AggregateBlock, path: &str) -> Option<ValidationError> {
    match aggregate.mode {
        AggregateMode::Join => {
            if aggregate.join.is_none() {
                return Some(ValidationError::new(
                    &format!("{}.aggregate", path),
                    "mode=join requires 'join' configuration with 'separator'",
                ));
            }
        }
        AggregateMode::Sum => {
            if aggregate.sum.is_none() {
                return Some(ValidationError::new(
                    &format!("{}.aggregate", path),
                    "mode=sum requires 'sum' expression field",
                ));
            }
        }
        _ => {}
    }
    None
}

/// Validate a motif file at the given path
pub fn validate_motif_file(path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let motif: MotifManifest = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse YAML: {}", path.display()))?;

    let errors = validate_motif(&motif);
    if !errors.is_empty() {
        for err in &errors {
            eprintln!("// Validation error in {}: {}", path.display(), err.message);
        }
        anyhow::bail!("Motif validation failed with {} error(s)", errors.len());
    }

    Ok(())
}

// ============================================================================
// Structure Manifest Validation
// ============================================================================

/// Validates a StructureManifest and returns a list of validation errors.
pub fn validate_structure(structure: &StructureManifest) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Validate structure has at least one motif
    if structure.motifs.is_empty() {
        errors.push(ValidationError::new(
            "motifs",
            "structure must have at least one motif reference",
        ));
    }

    // Validate motif references have names
    for (idx, motif_ref) in structure.motifs.iter().enumerate() {
        if motif_ref.name.is_empty() {
            errors.push(ValidationError::new(
                &format!("motifs[{}]", idx),
                "motif reference must have a name",
            ));
        }
    }

    errors
}

/// Validates that referenced motifs exist in the skills directory
pub fn validate_structure_motif_references(
    structure: &StructureManifest,
    skills: &SkillsDir,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    for motif_ref in &structure.motifs {
        if skills.find_motif(&motif_ref.name).is_none() {
            errors.push(ValidationError::new(
                &format!("motifs.{}", motif_ref.name),
                &format!("Referenced motif '{}' not found in skills/motifs/", motif_ref.name),
            ));
        }
    }

    errors
}

/// Validate a structure file at the given path
pub fn validate_structure_file(path: &Path, skills: &SkillsDir) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let structure: StructureManifest = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse YAML: {}", path.display()))?;

    // Validate structure manifest
    let errors = validate_structure(&structure);
    if !errors.is_empty() {
        for err in &errors {
            eprintln!("// Validation error in {}: {}", path.display(), err.message);
        }
        anyhow::bail!("Structure validation failed with {} error(s)", errors.len());
    }

    // Validate referenced motifs exist
    let ref_errors = validate_structure_motif_references(&structure, skills);
    if !ref_errors.is_empty() {
        for err in &ref_errors {
            eprintln!("// Reference error in {}: {}", path.display(), err.message);
        }
        anyhow::bail!("Structure references {} invalid motif(s)", ref_errors.len());
    }

    Ok(())
}

/// Auto-detect manifest type and validate
pub fn validate_manifest_file(path: &Path, skills: &SkillsDir) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    // Try to detect type from content
    if content.contains("type: structure") {
        validate_structure_file(path, skills)?;
    } else if content.contains("type: motif") || content.contains("flow:") {
        validate_motif_file(path)?;
    } else {
        anyhow::bail!("Cannot determine manifest type from content");
    }

    println!("// {} is valid", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::motif_manifest::{ErrorStrategy, MotifRef};
    use std::collections::HashMap;

    fn make_test_motif(flow: Vec<FlowStep>) -> MotifManifest {
        MotifManifest {
            name: "test-motif".to_string(),
            kind: "motif".to_string(),
            units_required: vec![],
            flow,
            return_expr: HashMap::new(),
        }
    }

    #[test]
    fn test_validate_motif_empty_flow() {
        let motif = make_test_motif(vec![]);
        let errors = validate_motif(&motif);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.message.contains("empty")));
    }

    #[test]
    fn test_validate_motif_step_unit_only() {
        let motif = make_test_motif(vec![FlowStep {
            name: "step1".to_string(),
            unit: Some("test-unit".to_string()),
            input: HashMap::new(),
            if_cond: None,
            foreach: None,
            on_error: None,
            fallback: None,
            retry: None,
            env_whitelist: None,
        }]);
        let errors = validate_motif(&motif);
        assert!(errors.is_empty(), "Valid motif should have no errors");
    }

    #[test]
    fn test_validate_motif_step_no_unit_no_foreach() {
        let motif = make_test_motif(vec![FlowStep {
            name: "step1".to_string(),
            unit: None,
            input: HashMap::new(),
            if_cond: None,
            foreach: None,
            on_error: None,
            fallback: None,
            retry: None,
            env_whitelist: None,
        }]);
        let errors = validate_motif(&motif);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.message.contains("must have either")));
    }

    #[test]
    fn test_validate_motif_retry_max_zero() {
        let motif = make_test_motif(vec![FlowStep {
            name: "step1".to_string(),
            unit: Some("test-unit".to_string()),
            input: HashMap::new(),
            if_cond: None,
            foreach: None,
            on_error: None,
            fallback: None,
            retry: Some(RetryConfig {
                max: 0,
                backoff: crate::engine::motif_manifest::BackoffStrategy::Exponential,
            }),
            env_whitelist: None,
        }]);
        let errors = validate_motif(&motif);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.message.contains("retry.max")));
    }

    #[test]
    fn test_validate_structure_empty_motifs() {
        let structure = StructureManifest {
            name: "test-structure".to_string(),
            kind: "structure".to_string(),
            motifs: vec![],
            input_schema: None,
            output_schema: None,
        };
        let errors = validate_structure(&structure);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.message.contains("at least one")));
    }

    #[test]
    fn test_validate_structure_valid() {
        let structure = StructureManifest {
            name: "test-structure".to_string(),
            kind: "structure".to_string(),
            motifs: vec![MotifRef {
                name: "test-motif".to_string(),
            }],
            input_schema: None,
            output_schema: None,
        };
        let errors = validate_structure(&structure);
        assert!(errors.is_empty(), "Valid structure should have no errors");
    }
}

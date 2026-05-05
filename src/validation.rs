use crate::discovery::SkillsDir;
use crate::engine::{MotifManifestV2, StructureManifest};
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::Path;
#[allow(unused_imports)]
use crate::engine::{Graph, MotifRef, Node, Edge};

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
// Motif Manifest Validation (JSON)
// ============================================================================

/// Validation error with detailed context
#[derive(Debug)]
pub struct ValidationError {
    pub message: String,
}

impl ValidationError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

/// Validates a MotifManifestV2 and returns a list of validation errors.
/// Empty list means validation passed.
pub fn validate_motif(motif: &MotifManifestV2) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Validate graph structure
    if let Err(e) = motif.graph.validate() {
        errors.push(ValidationError::new(&e.to_string()));
    }

    errors
}

/// Validate a motif file at the given path (JSON only)
pub fn validate_motif_file(path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let motif: MotifManifestV2 = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON: {}", path.display()))?;

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
// Structure Manifest Validation (JSON)
// ============================================================================

/// Validates a StructureManifest and returns a list of validation errors.
pub fn validate_structure(structure: &StructureManifest) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Validate structure has at least one motif
    if structure.motifs.is_empty() {
        errors.push(ValidationError::new(
            "structure must have at least one motif reference",
        ));
    }

    // Validate motif references have names
    for motif_ref in &structure.motifs {
        if motif_ref.name.is_empty() {
            errors.push(ValidationError::new(
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
                &format!("Referenced motif '{}' not found in skills/motifs/", motif_ref.name),
            ));
        }
    }

    errors
}

/// Validate a structure file at the given path (JSON only)
pub fn validate_structure_file(path: &Path, skills: &SkillsDir) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let structure: StructureManifest = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON: {}", path.display()))?;

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

/// Auto-detect manifest type and validate (JSON only)
pub fn validate_manifest_file(path: &Path, skills: &SkillsDir) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    // Try to detect type from content
    if content.contains("\"type\"") && content.contains("\"motifs\"") {
        validate_structure_file(path, skills)?;
    } else if content.contains("\"type\"") && content.contains("\"graph\"") {
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
    use crate::engine::{Graph, MotifRef, Node, Edge};
    use std::collections::HashMap;

    fn make_test_motif_valid() -> MotifManifestV2 {
        MotifManifestV2 {
            name: "test-motif".to_string(),
            kind: "motif".to_string(),
            version: None,
            description: None,
            required_units: vec![],
            graph: Graph {
                nodes: vec![
                    Node::Start { id: "start".to_string(), position: None },
                    Node::Return {
                        id: "ret".to_string(),
                        values: HashMap::new(),
                        position: None,
                    },
                ],
                edges: vec![
                    Edge {
                        id: None,
                        source: "start".to_string(),
                        target: "ret".to_string(),
                        label: None,
                        source_handle: None,
                        target_handle: None,
                    },
                ],
            },
            input_schema: None,
            output_schema: None,
        }
    }

    #[test]
    fn test_validate_motif_valid() {
        let motif = make_test_motif_valid();
        let errors = validate_motif(&motif);
        assert!(errors.is_empty(), "Valid motif should have no errors");
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

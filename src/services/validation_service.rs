//! Service layer for validation operations.
//! Wraps validation.rs functions for HTTP API use.

use crate::discovery::SkillsDir;
use crate::engine::motif_manifest::StructureManifest;
use crate::validation::{validate_motif, validate_structure, validate_structure_motif_references};
use anyhow::Result;
use std::path::Path;

/// Validation result with details.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

/// Validate a Structure manifest.
pub fn validate_structure_manifest(
    structure: &StructureManifest,
    skills: &SkillsDir,
) -> ValidationResult {
    let mut errors = validate_structure(structure);
    let ref_errors = validate_structure_motif_references(structure, skills);
    errors.extend(ref_errors);

    ValidationResult {
        valid: errors.is_empty(),
        errors: errors
            .into_iter()
            .map(|e| ValidationError {
                path: e.path,
                message: e.message,
            })
            .collect(),
    }
}

/// Validate a motif by name.
pub fn validate_motif_by_name(name: &str, skills: &SkillsDir) -> Result<ValidationResult> {
    let path = skills
        .find_motif(name)
        .ok_or_else(|| anyhow::anyhow!("Motif '{}' not found", name))?;

    let content = std::fs::read_to_string(&path)?;
    let motif: crate::engine::motif_manifest::MotifManifest =
        serde_yaml::from_str(&content)?;

    let errors = validate_motif(&motif);

    Ok(ValidationResult {
        valid: errors.is_empty(),
        errors: errors
            .into_iter()
            .map(|e| ValidationError {
                path: e.path,
                message: e.message,
            })
            .collect(),
    })
}

/// Validate a structure by name.
pub fn validate_structure_by_name(name: &str, skills: &SkillsDir) -> Result<ValidationResult> {
    let path = skills
        .find_structure(name)
        .ok_or_else(|| anyhow::anyhow!("Structure '{}' not found", name))?;

    let content = std::fs::read_to_string(&path)?;
    let structure: StructureManifest = serde_yaml::from_str(&content)?;

    Ok(validate_structure_manifest(&structure, skills))
}

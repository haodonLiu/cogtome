//! Service layer for validation operations.
//! Wraps validation.rs functions for HTTP API use.

use crate::discovery::SkillsDir;
use crate::engine::{MotifManifestV2, StructureManifest};
use anyhow::Result;

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
    _skills: &SkillsDir,
) -> ValidationResult {
    let mut errors = Vec::new();

    // Validate structure has at least one motif
    if structure.motifs.is_empty() {
        errors.push(ValidationError {
            path: "motifs".to_string(),
            message: "structure must have at least one motif reference".to_string(),
        });
    }

    // Validate motif references have names
    for (idx, motif_ref) in structure.motifs.iter().enumerate() {
        if motif_ref.name.is_empty() {
            errors.push(ValidationError {
                path: format!("motifs[{}]", idx),
                message: "motif reference must have a name".to_string(),
            });
        }
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
    }
}

/// Validate a motif manifest by parsing it.
pub fn validate_motif(motif: &MotifManifestV2) -> ValidationResult {
    let mut errors = Vec::new();

    // Validate graph
    if let Err(e) = motif.graph.validate() {
        errors.push(ValidationError {
            path: "graph".to_string(),
            message: e.to_string(),
        });
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
    }
}

/// Validate a motif by name.
pub fn validate_motif_by_name(name: &str, skills: &SkillsDir) -> Result<ValidationResult> {
    let path = skills
        .find_motif(name)
        .ok_or_else(|| anyhow::anyhow!("Motif '{}' not found", name))?;

    let content = std::fs::read_to_string(&path)?;
    let motif: MotifManifestV2 = serde_json::from_str(&content)?;

    Ok(validate_motif(&motif))
}

/// Validate a structure by name.
pub fn validate_structure_by_name(name: &str, skills: &SkillsDir) -> Result<ValidationResult> {
    let path = skills
        .find_structure(name)
        .ok_or_else(|| anyhow::anyhow!("Structure '{}' not found", name))?;

    let content = std::fs::read_to_string(&path)?;
    let structure: StructureManifest = serde_json::from_str(&content)?;

    Ok(validate_structure_manifest(&structure, skills))
}

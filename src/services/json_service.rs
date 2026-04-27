//! Service layer for JSON file operations.
//! Handles reading and writing JSON manifests.

use crate::engine::{MotifRef, MotifManifestV2, StructureManifest};
use anyhow::{Context, Result};
use std::path::Path;
use std::path::PathBuf;

/// Read a Structure manifest from the given path.
pub fn read_structure(path: &Path) -> Result<StructureManifest> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let manifest: StructureManifest = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON: {}", path.display()))?;

    Ok(manifest)
}

/// Write a Structure manifest to the given path.
pub fn write_structure(path: &Path, manifest: &StructureManifest) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let content = serde_json::to_string_pretty(manifest)
        .with_context(|| "Failed to serialize Structure manifest")?;

    std::fs::write(path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;

    Ok(())
}

/// Read a Motif manifest from the given path.
pub fn read_motif(path: &Path) -> Result<MotifManifestV2> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let manifest: MotifManifestV2 = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON: {}", path.display()))?;

    Ok(manifest)
}

/// Write a Motif manifest to the given path.
pub fn write_motif(path: &Path, manifest: &MotifManifestV2) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let content = serde_json::to_string_pretty(manifest)
        .with_context(|| "Failed to serialize Motif manifest")?;

    std::fs::write(path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;

    Ok(())
}

/// Resolve the path for a structure by name.
pub fn resolve_structure_path(skills_root: &Path, structures_subdir: &Path, name: &str) -> Option<PathBuf> {
    let path = skills_root.join(structures_subdir).join(name).join("manifest.json");
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

/// Resolve the path for a motif by name.
pub fn resolve_motif_path(skills_root: &Path, motifs_subdir: &Path, name: &str) -> Option<PathBuf> {
    let path = skills_root.join(motifs_subdir).join(format!("{}.json", name));
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

/// Check if a structure exists.
pub fn structure_exists(skills_root: &Path, structures_subdir: &Path, name: &str) -> bool {
    resolve_structure_path(skills_root, structures_subdir, name).is_some()
}

/// Get the directory path for a structure.
pub fn structure_dir(skills_root: &Path, structures_subdir: &Path, name: &str) -> PathBuf {
    skills_root.join(structures_subdir).join(name)
}

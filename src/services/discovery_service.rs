//! Service layer for file system discovery operations.
//! Extracts and reusable logic from discovery.rs for HTTP API use.

use crate::discovery::SkillsDir;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Information about a discovered Structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureInfo {
    pub name: String,
    pub path: PathBuf,
    pub motif_count: usize,
}

/// Information about a discovered Motif.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotifInfo {
    pub name: String,
    pub path: PathBuf,
    pub node_count: usize,
}

/// Information about a discovered Unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitInfo {
    pub name: String,
    pub path: PathBuf,
}

/// List all structures in the skills directory.
pub fn list_structures(skills: &SkillsDir) -> Result<Vec<StructureInfo>> {
    let structures_path = skills.root.join(&skills.structures_subdir);
    let mut structures = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&structures_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("manifest.json");
            if !manifest_path.exists() {
                continue;
            }

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Try to count motifs in the structure
            let motif_count = if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(val) => val.get("motifs")
                        .and_then(|v| v.as_array())
                        .map(|s| s.len())
                        .unwrap_or(0),
                    Err(_) => 0,
                }
            } else {
                0
            };

            structures.push(StructureInfo {
                name,
                path: manifest_path,
                motif_count,
            });
        }
    }

    // Sort by name
    structures.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(structures)
}

/// List all motifs in the skills directory (JSON only).
pub fn list_motifs(skills: &SkillsDir) -> Result<Vec<MotifInfo>> {
    let motifs_path = skills.root.join(&skills.motifs_subdir);
    let mut motifs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&motifs_path) {
        for entry in entries.flatten() {
            let path = entry.path();

            // Only consider .json files
            let ext = path.extension().and_then(|e| e.to_str());
            if !matches!(ext, Some("json")) {
                continue;
            }

            let name = path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Try to count nodes in the motif
            let node_count = if let Ok(content) = std::fs::read_to_string(&path) {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(val) => val.get("graph")
                        .and_then(|g| g.get("nodes"))
                        .and_then(|n| n.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0),
                    Err(_) => 0,
                }
            } else {
                0
            };

            motifs.push(MotifInfo {
                name,
                path,
                node_count,
            });
        }
    }

    // Sort by name
    motifs.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(motifs)
}

/// List all units in the skills directory.
pub fn list_units(skills: &SkillsDir) -> Result<Vec<UnitInfo>> {
    let units_path = skills.root.join(&skills.units_subdir);
    let mut units = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&units_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Unit executable is at <name>/bin/<name>
            let bin_path = path.join("bin").join(&name);
            if !bin_path.exists() {
                continue;
            }

            units.push(UnitInfo {
                name,
                path: bin_path,
            });
        }
    }

    // Sort by name
    units.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(units)
}

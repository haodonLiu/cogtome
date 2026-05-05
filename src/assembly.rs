//! Assembly - A publishable skill composed of Motifs and Units
//!
//! Assemblies are discovered from the assemblies/ directory.
//! Each assembly has a manifest.json that defines its metadata and workflow.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Assembly manifest from manifest.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyManifest {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(rename = "input_schema", default)]
    pub input_schema: Option<Value>,
    #[serde(rename = "output_schema", default)]
    pub output_schema: Option<Value>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub units: Vec<String>,
    pub workflow: String,
}

impl AssemblyManifest {
    /// Load manifest.json from an assembly directory
    pub fn load(dir: &Path) -> Result<Self> {
        let json_path = dir.join("manifest.json");
        let content = std::fs::read_to_string(&json_path)
            .with_context(|| format!("Failed to read manifest.json: {}", json_path.display()))?;
        let manifest: AssemblyManifest = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse manifest.json: {}", json_path.display()))?;
        Ok(manifest)
    }

    /// Get the workflow path (relative to assembly directory)
    pub fn workflow_path(&self, assembly_dir: &Path) -> PathBuf {
        assembly_dir.join(&self.workflow)
    }
}

/// Assembly - loaded assembly with resolved paths
#[derive(Debug, Clone)]
pub struct Assembly {
    pub manifest: AssemblyManifest,
    pub workflow_path: PathBuf,
}

impl Assembly {
    /// Load an assembly from its directory
    pub fn load(dir: PathBuf) -> Result<Self> {
        let manifest = AssemblyManifest::load(&dir)?;
        let workflow_path = manifest.workflow_path(&dir);

        Ok(Self {
            manifest,
            workflow_path,
        })
    }

    /// Get the assembly name
    pub fn name(&self) -> &str {
        &self.manifest.name
    }

    /// Get the workflow content as a string
    pub fn workflow_content(&self) -> Result<String> {
        std::fs::read_to_string(&self.workflow_path)
            .with_context(|| format!("Failed to read workflow: {}", self.workflow_path.display()))
    }
}

/// Assembly Registry - manages all available assemblies
#[derive(Clone)]
pub struct AssemblyRegistry {
    /// Base directory for assemblies
    base_dir: PathBuf,
    /// Map from assembly name to assembly
    assemblies: HashMap<String, Assembly>,
}

impl AssemblyRegistry {
    /// Create a new AssemblyRegistry by scanning the given directory
    pub fn new(base_dir: PathBuf) -> Result<Self> {
        let mut registry = Self {
            base_dir,
            assemblies: HashMap::new(),
        };
        registry.discover()?;
        Ok(registry)
    }

    /// Discover all assemblies in the base directory
    fn discover(&mut self) -> Result<()> {
        if !self.base_dir.exists() {
            warn!(dir = %self.base_dir.display(), "assemblies directory does not exist");
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.base_dir)?.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("manifest.json");
            if !manifest_path.exists() {
                debug!(dir = %path.display(), "skipping directory without manifest.json");
                continue;
            }

            match Assembly::load(path) {
                Ok(assembly) => {
                    // Verify workflow exists
                    if !assembly.workflow_path.exists() {
                        warn!(
                            assembly = %assembly.name(),
                            workflow = %assembly.workflow_path.display(),
                            "workflow file does not exist, skipping"
                        );
                        continue;
                    }

                    debug!(assembly = %assembly.name(), "discovered assembly");
                    self.assemblies.insert(assembly.name().to_string(), assembly);
                }
                Err(e) => {
                    warn!(path = %manifest_path.display(), error = %e, "failed to load assembly");
                }
            }
        }

        info!(count = self.assemblies.len(), "discovered assemblies");
        Ok(())
    }

    /// Find an assembly by name
    pub fn get(&self, name: &str) -> Option<&Assembly> {
        self.assemblies.get(name)
    }

    /// List all available assembly names
    pub fn list(&self) -> Vec<&str> {
        self.assemblies.keys().map(|s| s.as_str()).collect()
    }

    /// Get all assemblies
    pub fn all(&self) -> &HashMap<String, Assembly> {
        &self.assemblies
    }
}

/// Tool representation for MCP tools/list
#[derive(Debug, Clone, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Option<Value>,
}

impl From<&Assembly> for Tool {
    fn from(assembly: &Assembly) -> Self {
        Self {
            name: assembly.manifest.name.clone(),
            description: assembly.manifest.description.clone(),
            input_schema: assembly.manifest.input_schema.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_assembly_manifest_parsing() {
        let temp = tempfile::tempdir().unwrap();
        let asm_dir = temp.path().join("test-assembly");
        fs::create_dir_all(&asm_dir).unwrap();

        let manifest_json = r#"{
            "name": "test-assembly",
            "description": "A test assembly",
            "version": "1.0.0",
            "input_schema": {
                "type": "object",
                "properties": {
                    "text": {"type": "string"}
                }
            },
            "units": ["test-unit"],
            "workflow": "./workflow.json"
        }"#;
        fs::write(asm_dir.join("manifest.json"), manifest_json).unwrap();

        // Create workflow.json
        fs::write(asm_dir.join("workflow.json"), "{}").unwrap();

        let manifest = AssemblyManifest::load(&asm_dir).unwrap();
        assert_eq!(manifest.name, "test-assembly");
        assert_eq!(manifest.description, "A test assembly");
        assert!(manifest.input_schema.is_some());
        assert_eq!(manifest.units, vec!["test-unit"]);
    }
}

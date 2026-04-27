use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SkillsDir {
    pub root: PathBuf,
    pub units_subdir: PathBuf,
    pub motifs_subdir: PathBuf,
    pub structures_subdir: PathBuf,
}

impl Default for SkillsDir {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplexInfo {
    pub name: String,
    #[allow(dead_code)]
    pub path: PathBuf,
    pub description: String,
}

impl SkillsDir {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            units_subdir: PathBuf::from("units"),
            motifs_subdir: PathBuf::from("motifs"),
            structures_subdir: PathBuf::from("structures"),
        }
    }

    pub fn with_subdirs(root: PathBuf, units: PathBuf, motifs: PathBuf, structures: PathBuf) -> Self {
        Self {
            root,
            units_subdir: units,
            motifs_subdir: motifs,
            structures_subdir: structures,
        }
    }

    /// 查找 Unit 可执行文件：先全局，再各 Complex 私有
    pub fn find_unit(&self, name: &str) -> Option<PathBuf> {
        let global = self.root.join(&self.units_subdir).join(name).join("bin").join(name);
        if global.exists() && is_executable(&global) {
            return Some(global);
        }
        Self::scan_dirs(&self.root, |p| {
            let candidate = p.join(&self.units_subdir).join(name).join("bin").join(name);
            candidate.exists().then(|| candidate).filter(|c| is_executable(c))
        })
    }

    /// 查找 Motif 定义文件（仅 JSON 格式）
    pub fn find_motif(&self, name: &str) -> Option<PathBuf> {
        let global = self.root.join(&self.motifs_subdir).join(format!("{}.json", name));
        if global.exists() {
            return Some(global);
        }
        Self::scan_dirs(&self.root, |p| {
            let candidate = p.join(&self.motifs_subdir).join(format!("{}.json", name));
            candidate.exists().then_some(candidate)
        })
    }

    /// 查找 Structure manifest（仅 JSON 格式）
    pub fn find_structure(&self, name: &str) -> Option<PathBuf> {
        let global = self.root.join(&self.structures_subdir).join(name).join("manifest.json");
        if global.exists() {
            return Some(global);
        }
        Self::scan_dirs(&self.root, |p| {
            let candidate = p.join(&self.structures_subdir).join(name).join("manifest.json");
            candidate.exists().then_some(candidate)
        })
    }

    /// 扫描所有 Complex（有 SKILL.md 且含 description 的目录）
    pub fn discover_complexes(&self) -> Result<Vec<ComplexInfo>> {
        let mut complexes = Vec::new();
        for entry in std::fs::read_dir(&self.root)?.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let skill_md = path.join("SKILL.md");
            if !skill_md.exists() {
                continue;
            }
            let content = std::fs::read_to_string(&skill_md)?;
            if let Some(desc) = extract_description(&content) {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                complexes.push(ComplexInfo { name, path, description: desc });
            }
        }
        Ok(complexes)
    }

    fn scan_dirs<F>(root: &Path, mut finder: F) -> Option<PathBuf>
    where
        F: FnMut(&Path) -> Option<PathBuf>,
    {
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    if let Some(found) = finder(&entry.path()) {
                        return Some(found);
                    }
                }
            }
        }
        None
    }
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    // On non-Unix, just check if file exists (Windows uses extensions)
    path.exists()
}

/// Single structure entry in SKILL.md front matter
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StructureEntry {
    pub name: String,
    #[allow(dead_code)]
    pub path: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub summary: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub scenarios: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub weight: f64,
    #[serde(default)]
    #[allow(dead_code)]
    pub constraints: Vec<String>,
}

/// Root YAML front matter in SKILL.md
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkillMeta {
    pub description: String,
    #[serde(default)]
    pub structures: Vec<StructureEntry>,
}

/// Extracts and parses YAML front matter from SKILL.md content.
pub fn parse_skill_front_matter(content: &str) -> anyhow::Result<SkillMeta> {
    let trimmed = content.trim();
    anyhow::ensure!(trimmed.starts_with("---"), "Missing opening '---' delimiter");

    let after_first = &trimmed[3..];

    // Find closing --- at line boundary (not inside content)
    // Use join approach to correctly handle \r\n line endings
    let lines: Vec<&str> = after_first.lines().collect();
    let end_idx = lines
        .iter()
        .position(|l| l.trim() == "---")
        .ok_or_else(|| anyhow::anyhow!("Missing closing '---' delimiter"))?;

    let front_matter = lines[..end_idx].join("\n");
    let meta: SkillMeta = serde_yaml::from_str(&front_matter)
        .context("Failed to parse YAML front matter")?;
    Ok(meta)
}

/// Extracts description field from SKILL.md content.
pub fn extract_description(content: &str) -> Option<String> {
    parse_skill_front_matter(content).ok().map(|meta| meta.description)
}

/// Extracts first structure name from SKILL.md content.
pub fn extract_first_structure(content: &str) -> Option<String> {
    parse_skill_front_matter(content)
        .ok()
        .and_then(|meta| meta.structures.into_iter().next())
        .map(|s| s.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_front_matter_basic() {
        let content = r#"---
description: Test description
structures:
  - name: foo
    path: structures/foo
---
# Content
"#;
        let meta = parse_skill_front_matter(content).unwrap();
        assert_eq!(meta.description, "Test description");
        assert_eq!(meta.structures.len(), 1);
        assert_eq!(meta.structures[0].name, "foo");
    }

    #[test]
    fn test_front_matter_with_horizontal_rule_in_content() {
        // Content containing --- should not confuse the parser
        let content = r#"---
description: Test
---
Some text with --- separator inside it
"#;
        let meta = parse_skill_front_matter(content).unwrap();
        assert_eq!(meta.description, "Test");
    }

    #[test]
    fn test_missing_front_matter() {
        assert!(parse_skill_front_matter("No front matter").is_err());
    }

    #[test]
    fn test_missing_closing_delimiter() {
        let content = "---\ndescription: Test\n";
        assert!(parse_skill_front_matter(content).is_err());
    }

    #[test]
    fn test_extract_description_no_front_matter() {
        // Breaking change: no front matter means None
        assert!(extract_description("No front matter").is_none());
    }

    #[test]
    fn test_extract_first_structure() {
        let content = r#"---
description: Test
structures:
  - name: my-structure
    path: structures/my-structure
  - name: another
    path: structures/another
---
"#;
        assert_eq!(extract_first_structure(content), Some("my-structure".to_string()));
    }
}

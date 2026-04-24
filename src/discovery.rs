use anyhow::Result;
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

#[derive(Debug, Clone)]
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
        if global.exists() {
            return Some(global);
        }
        Self::scan_dirs(&self.root, |p| {
            let candidate = p.join(&self.units_subdir).join(name).join("bin").join(name);
            candidate.exists().then_some(candidate)
        })
    }

    /// 查找 Motif 定义文件
    pub fn find_motif(&self, name: &str) -> Option<PathBuf> {
        for ext in ["yaml", "yml", "py", "sh"] {
            let global = self.root.join(&self.motifs_subdir).join(format!("{}.{}", name, ext));
            if global.exists() {
                return Some(global);
            }
        }
        Self::scan_dirs(&self.root, |p| {
            let dir = p.join(&self.motifs_subdir);
            for ext in ["yaml", "yml", "py", "sh"] {
                let candidate = dir.join(format!("{}.{}", name, ext));
                if candidate.exists() {
                    return Some(candidate);
                }
            }
            None
        })
    }

    /// 查找 Structure manifest
    pub fn find_structure(&self, name: &str) -> Option<PathBuf> {
        let global = self.root.join(&self.structures_subdir).join(name).join("manifest.yaml");
        if global.exists() {
            return Some(global);
        }
        Self::scan_dirs(&self.root, |p| {
            let candidate = p.join(&self.structures_subdir).join(name).join("manifest.yaml");
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

/// 从 SKILL.md 内容中提取 description 字段
fn extract_description(content: &str) -> Option<String> {
    let trimmed = content.trim();
    if trimmed.starts_with("---") {
        if let Some(end) = trimmed[3..].find("---") {
            let front = &trimmed[3..3 + end];
            return extract_yaml_field(front, "description");
        }
    }
    extract_yaml_field(content, "description")
}

fn extract_yaml_field(text: &str, field: &str) -> Option<String> {
    let prefix = format!("{}:", field);
    let lines: Vec<&str> = text.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with(&prefix) {
            let val = trimmed[prefix.len()..].trim();
            // 多行 | 格式
            if val == "|" {
                let mut result = String::new();
                for j in (i + 1)..lines.len() {
                    let l = lines[j];
                    if l.starts_with("  ") || l.starts_with('\t') {
                        if !result.is_empty() {
                            result.push('\n');
                        }
                        result.push_str(l.trim_start_matches("  ").trim_start_matches('\t'));
                    } else if l.trim().is_empty() {
                        result.push('\n');
                    } else {
                        break;
                    }
                }
                return Some(result.trim().to_string());
            }
            return Some(
                val.trim_matches('"')
                    .trim_matches('\'')
                    .to_string(),
            );
        }
    }
    None
}

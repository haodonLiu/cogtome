use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct CogtomeConfig {
    #[serde(default)]
    #[allow(dead_code)]
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub paths: PathsConfig,
    #[serde(default)]
    pub units: UnitsConfig,
}

#[derive(Debug, Deserialize)]
pub struct RuntimeConfig {
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
    #[serde(default = "default_max_iterations_hard")]
    pub max_iterations_hard: u32,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_iterations: default_max_iterations(),
            max_iterations_hard: default_max_iterations_hard(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct PathsConfig {
    pub units: Option<String>,
    pub motifs: Option<String>,
    pub structures: Option<String>,
    #[serde(default)]
    pub assemblies: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UnitsConfig {
    #[serde(default)]
    pub defaults: UnitDefaults,
    #[serde(default)]
    pub concurrency: HashMap<String, ConcurrencyConfig>,
}

#[derive(Debug, Deserialize)]
pub struct UnitDefaults {
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

impl Default for UnitDefaults {
    fn default() -> Self {
        Self {
            timeout_secs: default_timeout(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ConcurrencyConfig {
    /// Max concurrent executions. -1 means unlimited.
    pub max_global: Option<i32>,
    #[allow(dead_code)]
    pub max_per_host: Option<i32>,
    pub resource_key: Option<String>,
}

fn default_max_iterations() -> u32 {
    50
}

fn default_max_iterations_hard() -> u32 {
    500
}

fn default_timeout() -> u64 {
    30
}

impl CogtomeConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: CogtomeConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// 查找配置文件：先当前目录，再 XDG 默认位置
    pub fn find() -> Option<PathBuf> {
        // 当前目录优先
        let local = PathBuf::from("./cogtome.toml");
        if local.exists() {
            return Some(local);
        }

        // XDG 默认位置 ($HOME/.config/cogtome.toml)
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            let xdg_path = PathBuf::from(xdg).join("cogtome.toml");
            if xdg_path.exists() {
                return Some(xdg_path);
            }
        }

        // HOME/.config/cogtome.toml
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(home).join(".config").join("cogtome.toml");
            if home_path.exists() {
                return Some(home_path);
            }
        }

        None
    }
}

impl Default for CogtomeConfig {
    fn default() -> Self {
        Self {
            runtime: RuntimeConfig::default(),
            paths: PathsConfig::default(),
            units: UnitsConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CogtomeConfig::default();
        assert_eq!(config.runtime.max_iterations, 50);
        assert_eq!(config.runtime.max_iterations_hard, 500);
        assert_eq!(config.units.defaults.timeout_secs, 30);
    }

    #[test]
    fn test_load_config() {
        let tmp_path = std::env::temp_dir().join("cogtome_test_config.toml");
        let toml_content = r#"
[runtime]
max_iterations = 100
max_iterations_hard = 1000

[paths]
units = "./my-units"

[units.defaults]
timeout_secs = 60

[units.concurrency.my-unit]
max_global = 5
resource_key = "api_quota"
"#;
        std::fs::write(&tmp_path, toml_content).unwrap();

        let config = CogtomeConfig::load(&tmp_path).unwrap();
        assert_eq!(config.runtime.max_iterations, 100);
        assert_eq!(config.runtime.max_iterations_hard, 1000);
        assert_eq!(config.paths.units.as_deref(), Some("./my-units"));
        assert_eq!(config.units.defaults.timeout_secs, 60);
        assert_eq!(config.units.concurrency.get("my-unit").unwrap().max_global, Some(5));

        std::fs::remove_file(tmp_path).ok();
    }

    #[test]
    fn test_load_config_minimal() {
        let tmp_path = std::env::temp_dir().join("cogtome_test_empty.toml");
        std::fs::write(&tmp_path, "").unwrap();
        let config = CogtomeConfig::load(&tmp_path).unwrap();
        // Should use defaults
        assert_eq!(config.runtime.max_iterations, 50);

        std::fs::remove_file(tmp_path).ok();
    }
}

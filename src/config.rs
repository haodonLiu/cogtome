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

#[derive(Debug, Deserialize, Default)]
pub struct RuntimeConfig {
    #[serde(default = "default_max_iterations")]
    #[allow(dead_code)]
    pub max_iterations: u32,
    #[serde(default = "default_max_iterations_hard")]
    #[allow(dead_code)]
    pub max_iterations_hard: u32,
}

#[derive(Debug, Deserialize, Default)]
pub struct PathsConfig {
    pub units: Option<String>,
    pub motifs: Option<String>,
    pub structures: Option<String>,
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
    pub max_global: Option<u32>,
    pub max_per_host: Option<u32>,
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

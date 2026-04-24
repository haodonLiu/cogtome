mod config;
mod context;
mod discovery;
mod engine;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::CogtomeConfig;
use discovery::{extract_first_structure, SkillsDir};
use engine::{StructureExecutor, UnitConcurrency, UnitRunner, YamlMotifEngine};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cogtome")]
#[command(about = "COGTOME - Agent Runtime Framework")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 管理 Unit（原子执行体）
    Unit {
        #[command(subcommand)]
        command: UnitCommands,
    },
    /// 管理 Motif（编排逻辑）
    Motif {
        #[command(subcommand)]
        command: MotifCommands,
    },
    /// 管理 Structure（业务结构）
    Structure {
        #[command(subcommand)]
        command: StructureCommands,
    },
    /// 运行 Complex（领域 Skill）
    Run {
        name: String,
        #[arg(short, long)]
        input: String,
    },
    /// 发现所有 Complex
    Discover,
}

#[derive(Subcommand)]
enum UnitCommands {
    /// 运行指定 Unit
    Run {
        name: String,
        #[arg(short, long)]
        input: String,
    },
}

#[derive(Subcommand)]
enum MotifCommands {
    /// 运行指定 Motif
    Run {
        name: String,
        #[arg(short, long)]
        input: String,
    },
}

#[derive(Subcommand)]
enum StructureCommands {
    /// 运行指定 Structure
    Run {
        name: String,
        #[arg(short, long)]
        input: String,
    },
}

fn resolve_skills_dir(config: &CogtomeConfig) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    // 环境变量 > 配置文件 > 默认值
    // paths.units 作为 root（向后兼容），motifs 和 structures 作为子目录覆盖
    let root = std::env::var("COGTOME_SKILLS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            config
                .paths
                .units
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("skills")
                })
        });

    // motifs 和 structures 路径：配置优先，否则使用相对于 root 的默认值
    let motifs = config
        .paths
        .motifs
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("motifs"));

    let structures = config
        .paths
        .structures
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("structures"));

    // units 子目录始终使用 "units"（向后兼容）
    let units = PathBuf::from("units");

    (root, units, motifs, structures)
}

fn resolve_timeout(config: &CogtomeConfig) -> u64 {
    std::env::var("COGTOME_TIMEOUT")
        .and_then(|v| v.parse().map_err(|_| std::env::VarError::NotPresent))
        .unwrap_or(config.units.defaults.timeout_secs)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 加载配置文件
    let config = match CogtomeConfig::find() {
        Some(path) => {
            eprintln!("// Loading config from {}", path.display());
            CogtomeConfig::load(&path)?
        }
        None => {
            eprintln!("// No config file found, using defaults");
            CogtomeConfig::default()
        }
    };

    let (root, units, motifs, structures) = resolve_skills_dir(&config);
    let skills = SkillsDir::with_subdirs(root, units, motifs, structures);
    let timeout = resolve_timeout(&config);
    let concurrency_config: HashMap<String, UnitConcurrency> = config
        .units
        .concurrency
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                UnitConcurrency {
                    max_global: v.max_global,
                    max_per_host: v.max_per_host,
                    resource_key: v.resource_key,
                },
            )
        })
        .collect();
    let runner = UnitRunner::new_with_config(skills.clone(), timeout, concurrency_config);

    match cli.command {
        // ------------------------------------------------------------------
        // Unit 层：直接调用原子执行体
        // ------------------------------------------------------------------
        Commands::Unit { command } => match command {
            UnitCommands::Run { name, input } => {
                let val: Value = serde_json::from_str(&input)?;
                let (result, exit_code) = runner.call(&name, val).await?;
                println!("{}", serde_json::to_string_pretty(&result)?);
                eprintln!("[exit code: {}]", exit_code);
            }
        },

        // ------------------------------------------------------------------
        // Motif 层：执行编排逻辑
        // ------------------------------------------------------------------
        Commands::Motif { command } => match command {
            MotifCommands::Run { name, input } => {
                let val: Value = serde_json::from_str(&input)?;
                let path = skills
                    .find_motif(&name)
                    .ok_or_else(|| anyhow::anyhow!("Motif '{}' not found", name))?;
                let manifest = YamlMotifEngine::load(&path)?;
                let engine = YamlMotifEngine;
                let result = engine.execute(&manifest, val, &runner).await?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        },

        // ------------------------------------------------------------------
        // Structure 层：加载业务结构并执行
        // ------------------------------------------------------------------
        Commands::Structure { command } => match command {
            StructureCommands::Run { name, input } => {
                let val: Value = serde_json::from_str(&input)?;
                let path = skills
                    .find_structure(&name)
                    .ok_or_else(|| anyhow::anyhow!("Structure '{}' not found", name))?;
                let manifest = StructureExecutor::load(&path)?;
                let result =
                    StructureExecutor::execute(&manifest, val, &skills, &runner).await?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        },

        // ------------------------------------------------------------------
        // Complex 层：发现 → 选择 Structure → 执行
        // ------------------------------------------------------------------
        Commands::Run { name, input } => {
            let val: Value = serde_json::from_str(&input)?;

            // 1. 定位 Complex
            let complex_path = skills.root.join(&name);
            if !complex_path.exists() {
                anyhow::bail!("Complex '{}' not found at {}", name, complex_path.display());
            }

            // 2. 读取 SKILL.md，提取第一个 Structure（简化版选择器）
            let skill_md = std::fs::read_to_string(complex_path.join("SKILL.md"))?;
            let structure_name = extract_first_structure(&skill_md)
                .ok_or_else(|| anyhow::anyhow!("No structure found in Complex '{}'", name))?;

            println!("// Complex: {} → Structure: {}", name, structure_name);

            // 3. 执行 Structure
            let path = skills.find_structure(&structure_name).ok_or_else(|| {
                anyhow::anyhow!("Structure '{}' not found", structure_name)
            })?;
            let manifest = StructureExecutor::load(&path)?;
            let result = StructureExecutor::execute(&manifest, val, &skills, &runner).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        // ------------------------------------------------------------------
        // 发现：扫描所有 Complex
        // ------------------------------------------------------------------
        Commands::Discover => {
            let complexes = skills.discover_complexes()?;
            if complexes.is_empty() {
                println!("No Complexes found in {}", skills.root.display());
            } else {
                println!("Found {} Complex(es):\n", complexes.len());
                for c in complexes {
                    let desc = c.description.lines().next().unwrap_or("");
                    println!("  {:20} {}", c.name, desc);
                }
            }
        }
    }

    Ok(())
}

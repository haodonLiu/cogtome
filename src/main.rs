mod context;
mod discovery;
mod engine;

use anyhow::Result;
use clap::{Parser, Subcommand};
use discovery::SkillsDir;
use engine::{StructureExecutor, UnitRunner, YamlMotifEngine};
use serde_json::Value;
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

fn skills_dir() -> SkillsDir {
    let path = std::env::var("COGTOME_SKILLS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("skills"));
    SkillsDir::new(path)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let skills = skills_dir();
    let runner = UnitRunner::new(skills.clone(), 30); // 30s default timeout

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

/// 从 SKILL.md 中提取 structures 列表下的第一个 structure 名称
fn extract_first_structure(content: &str) -> Option<String> {
    let mut in_structures = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "structures:" {
            in_structures = true;
            continue;
        }
        if in_structures {
            if trimmed.starts_with("- name:") {
                return Some(trimmed[7..].trim().to_string());
            }
            // 遇到非缩进行且不是 name 开头，说明 structures 块结束
            if !trimmed.is_empty() && !trimmed.starts_with("-") && !trimmed.starts_with("name:") && !trimmed.starts_with("path:") && !trimmed.starts_with("summary:") && !trimmed.starts_with("scenarios:") && !trimmed.starts_with("weight:") && !trimmed.starts_with("constraints:") {
                // 继续扫描，因为可能有其他字段
                continue;
            }
        }
    }
    None
}

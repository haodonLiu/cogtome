mod api;
mod assembly;
mod config;
mod context;
mod discovery;
mod engine;
mod error;
mod metrics;
mod mcp_server;
mod pack;
mod services;
mod shutdown;
mod validation;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::CogtomeConfig;
use discovery::{extract_first_structure, SkillsDir};
use engine::{GraphMotifEngine, McpBridgeInput, McpBridgeUnit, StructureExecutor, UnitConcurrency, UnitRunner};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

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
    /// 启动 HTTP API 服务器
    Serve {
        #[arg(long, default_value = "3334")]
        port: u16,
    },
    /// 打包 Skill 到 .cogtome 归档
    Pack {
        name: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// 安装 .cogtome 归档
    Install {
        path: String,
    },
    /// 热重载：重新加载所有 Structure 和 Motif 定义
    Reload,
    /// 验证 Motif 或 Structure manifest 文件
    Validate {
        path: String,
    },
    /// 通过 MCP Bridge 运行 MCP Server 工具
    McpBridge {
        /// MCP Server 启动命令，如 "npx -y @modelcontextprotocol/server-filesystem /tmp"
        #[arg(long)]
        server: String,
        /// 要调用的工具名，如 "read_text_file"
        #[arg(long)]
        tool: String,
        /// 工具参数（JSON 格式）
        #[arg(long, default_value = "{}")]
        args: String,
        /// 初始化超时（秒）
        #[arg(long, default_value = "30")]
        init_timeout: u64,
        /// 请求超时（秒）
        #[arg(long, default_value = "60")]
        request_timeout: u64,
    },
    /// 启动 MCP Server（stdio 模式）
    McpServer {
        /// Assemblies 目录
        #[arg(long, default_value = "./assemblies")]
        assemblies: String,
        /// Units 目录
        #[arg(long, default_value = "./units")]
        units: String,
        /// 执行超时（秒）
        #[arg(long, default_value = "30")]
        timeout: u64,
    },
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

fn resolve_timeout(config: &CogtomeConfig) -> u64 {
    std::env::var("COGTOME_TIMEOUT")
        .and_then(|v| v.parse().map_err(|_| std::env::VarError::NotPresent))
        .unwrap_or(config.units.defaults.timeout_secs)
}

fn resolve_max_iterations_hard(config: &CogtomeConfig) -> u32 {
    config.runtime.max_iterations_hard
}

#[derive(Debug, Clone)]
struct SkillsPaths {
    root: PathBuf,
    units: PathBuf,
    motifs: PathBuf,
    structures: PathBuf,
}

fn resolve_skills_dir(config: &CogtomeConfig) -> SkillsPaths {
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
    // Note: If config paths are absolute, they override root (user intent is respected)
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

    SkillsPaths { root, units, motifs, structures }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber
    let log_format = std::env::var("COGTOME_LOG_FORMAT")
        .unwrap_or_else(|_| "pretty".to_string());
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    match log_format.as_str() {
        "json" => {
            tracing_subscriber::fmt()
                .with_env_filter(&log_level)
                .json()
                .init();
        }
        _ => {
            tracing_subscriber::fmt()
                .with_env_filter(&log_level)
                .pretty()
                .init();
        }
    }

    tracing::info!("cogtome starting up");

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

    let paths = resolve_skills_dir(&config);
    let skills_root = paths.root.clone();
    let skills = SkillsDir::with_subdirs(paths.root, paths.units, paths.motifs, paths.structures);
    let timeout = resolve_timeout(&config);
    let max_iterations_hard = resolve_max_iterations_hard(&config);
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
                let (result, exit_code) = runner.call(&name, val, None).await?;
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

                let manifest = GraphMotifEngine::load(&path)?;
                let engine = GraphMotifEngine;
                let result = engine.execute(&manifest, val, &runner, max_iterations_hard).await?;

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
                    StructureExecutor::execute(&manifest, val, &skills, &runner, max_iterations_hard).await?;
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
            let result = StructureExecutor::execute(&manifest, val, &skills, &runner, max_iterations_hard).await?;
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

        // ------------------------------------------------------------------
        // HTTP API 服务器
        // ------------------------------------------------------------------
        Commands::Serve { port } => {
            let graceful = shutdown::GracefulShutdown::new();
            let token = graceful.token();

            api::start_server_with_shutdown(port, skills.clone(), timeout, token).await?;

            // Log graceful shutdown completion
            if graceful.is_shutdown_requested() {
                tracing::info!("HTTP server shutdown complete");
            }
        }

        // ------------------------------------------------------------------
        // 打包 Skill
        // ------------------------------------------------------------------
        Commands::Pack { name, output } => {
            let output_path = output.map(PathBuf::from);
            let packed = pack::pack(&name, &skills_root, output_path)?;
            println!("Packed to {}", packed.display());
        }

        // ------------------------------------------------------------------
        // 安装 Skill
        // ------------------------------------------------------------------
        Commands::Install { path } => {
            pack::install(PathBuf::from(&path).as_path(), &skills_root)?;
            println!("Installed successfully");
        }

        // ------------------------------------------------------------------
        // 热重载：重新加载所有 Structure 和 Motif 定义
        // ------------------------------------------------------------------
        Commands::Reload => {
            // Re-discover all complexes to validate they still exist and are valid
            let complexes = skills.discover_complexes()?;

            // Count structures and motifs by walking the skills directory
            let mut structure_count = 0;
            let mut motif_count = 0;

            let structures_path = skills.root.join(&skills.structures_subdir);
            if let Ok(entries) = std::fs::read_dir(&structures_path) {
                for entry in entries.flatten() {
                    if entry.path().join("manifest.json").exists() {
                        structure_count += 1;
                    }
                }
            }

            let motifs_path = skills.root.join(&skills.motifs_subdir);
            if let Ok(entries) = std::fs::read_dir(&motifs_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "json").unwrap_or(false) {
                        motif_count += 1;
                    }
                }
            }

            println!("Registry reloaded:");
            println!("  Complexes: {}", complexes.len());
            println!("  Structures: {}", structure_count);
            println!("  Motifs: {}", motif_count);
        }

        // ------------------------------------------------------------------
        // 验证：检查 Motif 或 Structure manifest 文件
        // ------------------------------------------------------------------
        Commands::Validate { path } => {
            let path = PathBuf::from(&path);
            if !path.exists() {
                anyhow::bail!("File not found: {}", path.display());
            }
            validation::validate_manifest_file(&path, &skills)?;
        }

        // ------------------------------------------------------------------
        // MCP Bridge：运行 MCP Server 工具
        // ------------------------------------------------------------------
        Commands::McpBridge { server, tool, args, init_timeout, request_timeout } => {
            let args_value: Value = serde_json::from_str(&args).map_err(|e| anyhow::anyhow!("Invalid args JSON: {}", e))?;
            let args_map = if let Value::Object(map) = args_value {
                map.into_iter().collect()
            } else {
                anyhow::bail!("args must be a JSON object");
            };

            let input = McpBridgeInput {
                server,
                tool,
                args: args_map,
                init_timeout,
                request_timeout,
            };

            let result = McpBridgeUnit::execute(input).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        // ------------------------------------------------------------------
        // MCP Server：启动 MCP Server（stdio 模式）
        // ------------------------------------------------------------------
        Commands::McpServer { assemblies, units, timeout } => {
            use mcp_server::run_server;

            let assemblies_dir = PathBuf::from(assemblies);
            let units_dir = PathBuf::from(units);

            info!(
                assemblies = %assemblies_dir.display(),
                units = %units_dir.display(),
                timeout = timeout,
                "starting MCP server"
            );

            run_server(assemblies_dir, units_dir, timeout)?;
        }
    }

    Ok(())
}

use crate::context::ExecContext;
use crate::discovery::SkillsDir;
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

// ============================================================================
// Unit Runner
// ============================================================================

#[derive(Debug, Clone)]
pub struct UnitRunner {
    skills: SkillsDir,
}

impl UnitRunner {
    pub fn new(skills: SkillsDir) -> Self {
        Self { skills }
    }

    /// 调用 Unit，返回 (stdout_json, exit_code)
    pub async fn call(&self, name: &str, input: Value) -> Result<(Value, i32)> {
        let bin_path = self
            .skills
            .find_unit(name)
            .with_context(|| format!("Unit '{}' not found", name))?;

        let mut child = Command::new(&bin_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .env("COGTOME_UNIT_MODE", "1")
            .spawn()
            .with_context(|| format!("Failed to spawn unit '{}'", name))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input.to_string().as_bytes()).await?;
        }

        let output = child.wait_with_output().await?;
        let exit_code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        if !output.status.success() {
            anyhow::bail!(
                "Unit '{}' exited with code {}: {}",
                name,
                exit_code,
                stderr.trim()
            );
        }

        let result: Value = serde_json::from_str(&stdout).with_context(|| {
            format!(
                "Invalid JSON output from unit '{}': {}",
                name,
                stdout.trim()
            )
        })?;

        Ok((result, exit_code))
    }
}

// ============================================================================
// Motif Engine (YAML)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct MotifManifest {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub units_required: Vec<String>,
    pub flow: Vec<FlowStep>,
    #[serde(default, rename = "return")]
    pub return_expr: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct FlowStep {
    pub name: String,
    pub unit: String,
    pub input: HashMap<String, String>,
}

pub struct YamlMotifEngine;

impl YamlMotifEngine {
    pub fn load(path: &Path) -> Result<MotifManifest> {
        let content = std::fs::read_to_string(path)?;
        let manifest: MotifManifest = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse motif manifest: {}", path.display()))?;
        Ok(manifest)
    }

    pub async fn execute(
        &self,
        manifest: &MotifManifest,
        input: Value,
        runner: &UnitRunner,
    ) -> Result<Value> {
        let mut ctx = ExecContext::new(input);

        for step in &manifest.flow {
            // 构建步骤输入：解析模板变量
            let mut step_input = serde_json::Map::new();
            for (k, v) in &step.input {
                let val = ctx.resolve_var(v).unwrap_or(Value::Null);
                step_input.insert(k.clone(), val);
            }

            let (output, exit_code) = runner
                .call(&step.unit, Value::Object(step_input))
                .await?;

            ctx.steps.insert(
                step.name.clone(),
                crate::context::StepResult {
                    output,
                    exit_code,
                },
            );
        }

        // 构建 return 对象
        let mut result = serde_json::Map::new();
        for (k, v) in &manifest.return_expr {
            if let Some(val) = ctx.resolve_var(v) {
                result.insert(k.clone(), val);
            }
        }

        Ok(Value::Object(result))
    }
}

// ============================================================================
// Structure Executor
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct StructureManifest {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub motifs: Vec<MotifRef>,
    #[serde(default)]
    pub input_schema: Option<Value>,
    #[serde(default)]
    pub output_schema: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct MotifRef {
    pub name: String,
}

pub struct StructureExecutor;

impl StructureExecutor {
    pub fn load(path: &Path) -> Result<StructureManifest> {
        let content = std::fs::read_to_string(path)?;
        let manifest: StructureManifest = serde_yaml::from_str(&content).with_context(|| {
            format!(
                "Failed to parse structure manifest: {}",
                path.display()
            )
        })?;
        Ok(manifest)
    }

    pub async fn execute(
        manifest: &StructureManifest,
        input: Value,
        skills: &SkillsDir,
        runner: &UnitRunner,
    ) -> Result<Value> {
        let mut current = input;

        for motif_ref in &manifest.motifs {
            let motif_path = skills
                .find_motif(&motif_ref.name)
                .ok_or_else(|| anyhow::anyhow!("Motif '{}' not found", motif_ref.name))?;

            let motif_manifest = YamlMotifEngine::load(&motif_path)?;
            let engine = YamlMotifEngine;
            current = engine.execute(&motif_manifest, current, runner).await?;
        }

        Ok(current)
    }
}

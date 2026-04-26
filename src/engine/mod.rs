pub mod foreach;
pub mod motif_manifest;
pub mod unit_runner;

pub use foreach::{
    emit_step_end, emit_step_start, execute_foreach_parallel, execute_foreach_serial,
    execute_unit_with_error_handling, resolve_step_input,
};
#[allow(unused_imports)]
pub use motif_manifest::{
    AggregateBlock, AggregateMode, BackoffStrategy, ErrorStrategy, FlowStep, ForeachBlock,
    JoinConfig, MotifManifest, MotifRef, RetryConfig, StepErrorStrategy, StructureManifest,
};
pub use unit_runner::{UnitConcurrency, UnitRunner};

use crate::context::{is_truthy, ExecContext, StepResult};
use crate::discovery::SkillsDir;
use crate::error::{CogtomeError, ErrorCode, ErrorLayer};
use crate::validation::validate_input;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use tracing::{error, info, warn, Instrument};

// ============================================================================
// Motif Engine (YAML)
// ============================================================================

#[derive(Clone)]
pub struct YamlMotifEngine;

impl YamlMotifEngine {
    pub fn load(path: &Path) -> Result<MotifManifest> {
        let content = std::fs::read_to_string(path).with_context(|| {
            format!("Failed to read motif manifest: {}", path.display())
        })?;
        let manifest: MotifManifest = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse motif manifest: {}", path.display()))?;

        for step in &manifest.flow {
            if step.unit.is_some() && step.foreach.is_some() {
                return Err(CogtomeError::new(
                    ErrorLayer::Motif,
                    ErrorCode::EMotifParse,
                    format!("FlowStep '{}' has both 'unit' and 'foreach' - they are mutually exclusive", step.name),
                ).into());
            }
            if step.unit.is_none() && step.foreach.is_none() {
                return Err(CogtomeError::new(
                    ErrorLayer::Motif,
                    ErrorCode::EMotifParse,
                    format!("FlowStep '{}' must have either 'unit' or 'foreach'", step.name),
                ).into());
            }
        }

        Ok(manifest)
    }

    pub async fn execute(
        &self,
        manifest: &MotifManifest,
        input: Value,
        runner: &UnitRunner,
        max_iterations_hard: u32,
    ) -> Result<Value> {
        let span = tracing::info_span!(
            "motif",
            motif.name = %manifest.name,
            motif.step_count = manifest.flow.len()
        );

        async move {
            let mut ctx = ExecContext::new(input);

            for step in &manifest.flow {
                let step_span = tracing::info_span!("step", step.name = %step.name);
                let _guard = step_span.enter();

                if let Some(cond) = &step.if_cond {
                    let val = ctx.resolve_var(cond).unwrap_or(Value::Null);
                    if !is_truthy(&val) {
                        warn!(step = %step.name, condition = %cond, "step skipped by condition");
                        continue;
                    }
                }

                if let Some(foreach) = &step.foreach {
                    info!(step = %step.name, foreach.over = %foreach.over, foreach.parallel = foreach.parallel);
                    let result = if foreach.parallel {
                        execute_foreach_parallel(foreach, &ctx, runner, max_iterations_hard).await?
                    } else {
                        execute_foreach_serial(foreach, &ctx, runner, max_iterations_hard).await?
                    };
                    ctx = ctx.with_local_step(
                        step.name.clone(),
                        StepResult {
                            output: result,
                            exit_code: 0,
                        },
                    );
                    continue;
                }

                if let Some(unit_name) = &step.unit {
                    emit_step_start(&step.name, Some(unit_name));
                    let start = Instant::now();

                    let step_input = resolve_step_input(&ctx, &step.input)?;
                    let result = execute_unit_with_error_handling(
                        runner,
                        unit_name,
                        Value::Object(step_input),
                        &step.retry,
                        &step.on_error,
                        &step.fallback,
                        &step.env_whitelist,
                    )
                    .await;

                    let duration_ms = start.elapsed().as_millis() as u64;
                    match result {
                        Ok((output, exit_code, _)) => {
                            emit_step_end(&step.name, duration_ms, "ok", Some(exit_code));
                            ctx = ctx.with_local_step(
                                step.name.clone(),
                                StepResult { output, exit_code },
                            );
                        }
                        Err(e) => {
                            emit_step_end(&step.name, duration_ms, "error", None);
                            error!(step = %step.name, error = %e, "step failed");
                            return Err(e);
                        }
                    }
                }
            }

            let result = self.build_return(&manifest.return_expr, &ctx)?;
            info!(motif = %manifest.name, result_keys = result.as_object().map(|o| o.len()).unwrap_or(0), "motif completed");
            Ok(result)
        }
        .instrument(span)
        .await
    }

    fn build_return(
        &self,
        return_expr: &HashMap<String, String>,
        ctx: &ExecContext,
    ) -> Result<Value> {
        let mut result = serde_json::Map::new();
        for (k, v) in return_expr {
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

fn validate_structure_input(manifest: &StructureManifest, input: &Value) -> Result<()> {
    if let Some(ref schema) = manifest.input_schema {
        validate_input(input, schema)?;
    }
    Ok(())
}

pub struct StructureExecutor;

impl StructureExecutor {
    pub fn load(path: &Path) -> Result<StructureManifest> {
        let content = std::fs::read_to_string(path).with_context(|| {
            format!("Failed to read structure manifest: {}", path.display())
        })?;
        let manifest: StructureManifest = serde_yaml::from_str(&content).with_context(|| {
            format!("Failed to parse structure manifest: {}", path.display())
        })?;
        Ok(manifest)
    }

    pub async fn execute(
        manifest: &StructureManifest,
        input: Value,
        skills: &SkillsDir,
        runner: &UnitRunner,
        max_iterations_hard: u32,
    ) -> Result<Value> {
        let span = tracing::info_span!(
            "structure",
            structure.name = %manifest.name,
            motif_count = manifest.motifs.len()
        );

        async move {
            validate_structure_input(manifest, &input)?;

            let mut current = input;

            for motif_ref in &manifest.motifs {
                let motif_path = skills.find_motif(&motif_ref.name).ok_or_else(|| {
                    error!(motif = %motif_ref.name, "motif not found");
                    CogtomeError::new(
                        ErrorLayer::Motif,
                        ErrorCode::EMotifNotFound,
                        format!("Motif '{}' not found", motif_ref.name),
                    )
                    .with_hint("Ensure the motif is defined in skills/motifs/<name>.yaml")
                })?;

                let motif_manifest = YamlMotifEngine::load(&motif_path)?;
                let engine = YamlMotifEngine;
                info!(structure = %manifest.name, motif = %motif_ref.name);
                current = engine
                    .execute(&motif_manifest, current, runner, max_iterations_hard)
                    .await?;
            }

            info!(structure = %manifest.name, "structure completed");
            Ok(current)
        }
        .instrument(span)
        .await
    }
}

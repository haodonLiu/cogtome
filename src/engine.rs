use crate::context::{is_truthy, ExecContext, StepResult};
use crate::discovery::SkillsDir;
use crate::validation::validate_input;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::{Semaphore, Mutex};
use tokio_util::sync::CancellationToken;

// ============================================================================
// Unit Concurrency
// ============================================================================

#[derive(Debug, Clone)]
pub struct UnitConcurrency {
    pub max_global: Option<u32>,
    #[allow(dead_code)]
    pub max_per_host: Option<u32>,
    pub resource_key: Option<String>,
}

// ============================================================================
// Unit Runner
// ============================================================================

#[derive(Clone)]
pub struct UnitRunner {
    skills: SkillsDir,
    timeout_secs: u64,
    concurrency_config: HashMap<String, UnitConcurrency>,
    resource_semaphores: Arc<HashMap<String, Arc<Semaphore>>>,
    undeclared_semaphores: Arc<Mutex<HashMap<String, Arc<Semaphore>>>>,
}

impl UnitRunner {
    pub fn new_with_config(skills: SkillsDir, timeout_secs: u64, concurrency_config: HashMap<String, UnitConcurrency>) -> Self {
        // Build resource semaphores from config
        let mut resource_semaphores: HashMap<String, Arc<Semaphore>> = HashMap::new();
        for (_unit_name, config) in &concurrency_config {
            if let Some(ref key) = config.resource_key {
                // Use max_global as semaphore capacity, default to 1 if not set
                let permits = config.max_global.unwrap_or(1);
                resource_semaphores.insert(key.clone(), Arc::new(Semaphore::new(permits as usize)));
            }
        }

        Self {
            skills,
            timeout_secs,
            concurrency_config,
            resource_semaphores: Arc::new(resource_semaphores),
            undeclared_semaphores: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn call(&self, name: &str, input: Value) -> Result<(Value, i32)> {
        // Acquire semaphore permit for rate limiting
        let sem = self.get_semaphore(name).await;
        let _permit = sem.acquire().await.map_err(|e| anyhow!("Semaphore error: {}", e))?;

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

        // Use Arc<Mutex<Option<Child>>> to allow taking child for kill on timeout
        let child_arc = Arc::new(tokio::sync::Mutex::new(Some(child)));

        let child_for_kill = child_arc.clone();
        let output = tokio::time::timeout(
            Duration::from_secs(self.timeout_secs),
            async {
                let mut guard = child_arc.lock().await;
                if let Some(child) = guard.take() {
                    child.wait_with_output().await
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "child already taken",
                    ))
                }
            },
        )
        .await;

        let output = match output {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => anyhow::bail!("Unit '{}' I/O error: {}", name, e),
            Err(_) => {
                // Timeout - kill the child process
                let mut guard = child_for_kill.lock().await;
                if let Some(mut child) = guard.take() {
                    let _ = child.kill().await;
                }
                anyhow::bail!("Unit '{}' timed out after {}s", name, self.timeout_secs);
            }
        };

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

    async fn get_semaphore(&self, unit_name: &str) -> Arc<Semaphore> {
        // Check if unit has explicit concurrency config with resource_key
        if let Some(concurrency) = self.concurrency_config.get(unit_name) {
            if let Some(ref key) = concurrency.resource_key {
                if let Some(sem) = self.resource_semaphores.get(key) {
                    return sem.clone();
                }
            }
        }

        // Lazy init for undeclared units (capacity 1 = serialized per unit)
        let sem = {
            let mut map = self.undeclared_semaphores.lock().await;
            map.entry(unit_name.to_string())
                .or_insert_with(|| Arc::new(Semaphore::new(1)))
                .clone()
        };

        sem
    }
}

// ============================================================================
// Motif Engine (YAML)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct MotifManifest {
    #[allow(dead_code)]
    pub name: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub kind: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub units_required: Vec<String>,
    pub flow: Vec<FlowStep>,
    #[serde(default, rename = "return")]
    pub return_expr: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FlowStep {
    pub name: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub input: HashMap<String, String>,
    #[serde(default, rename = "if")]
    pub if_cond: Option<String>,
    #[serde(default)]
    pub foreach: Option<ForeachBlock>,
    #[serde(default)]
    pub on_error: Option<StepErrorStrategy>,
    #[serde(default)]
    pub fallback: Option<Value>,
    #[serde(default)]
    pub retry: Option<RetryConfig>,
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepErrorStrategy {
    #[default]
    Fail,
    Continue,
    Fallback,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RetryConfig {
    pub max: u32,
    #[serde(default = "default_backoff")]
    pub backoff: BackoffStrategy,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BackoffStrategy {
    #[default]
    Exponential,
    Linear,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ForeachBlock {
    pub over: String,
    #[serde(default = "default_as_var")]
    pub as_var: String,
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
    #[serde(default = "default_on_error")]
    pub on_error: ErrorStrategy,
    #[serde(default = "default_parallel")]
    pub parallel: bool,
    pub flow: Vec<FlowStep>,
    pub aggregate: AggregateBlock,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AggregateBlock {
    pub mode: AggregateMode,
    #[serde(default)]
    pub map: HashMap<String, String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub sum: Option<String>,
    #[serde(default)]
    pub join: Option<JoinConfig>,
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorStrategy {
    #[default]
    FailFast,
    Continue,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AggregateMode {
    Array,
    Object,
    Sum,
    Join,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JoinConfig {
    #[allow(dead_code)]
    pub expr: String,
    #[serde(default)]
    pub separator: String,
}

fn default_as_var() -> String {
    "item".to_string()
}

fn default_max_iterations() -> u32 {
    50
}

fn default_on_error() -> ErrorStrategy {
    ErrorStrategy::FailFast
}

fn default_parallel() -> bool {
    false
}

fn default_backoff() -> BackoffStrategy {
    BackoffStrategy::Exponential
}

pub struct YamlMotifEngine;

impl YamlMotifEngine {
    pub fn load(path: &Path) -> Result<MotifManifest> {
        let content = std::fs::read_to_string(path)?;
        let manifest: MotifManifest = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse motif manifest: {}", path.display()))?;

        // Validate FlowStep mutual exclusivity
        for step in &manifest.flow {
            if step.unit.is_some() && step.foreach.is_some() {
                anyhow::bail!(
                    "FlowStep '{}' has both 'unit' and 'foreach' - they are mutually exclusive",
                    step.name
                );
            }
            if step.unit.is_none() && step.foreach.is_none() {
                anyhow::bail!(
                    "FlowStep '{}' must have either 'unit' or 'foreach'",
                    step.name
                );
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
        let mut ctx = ExecContext::new(input);

        for step in &manifest.flow {
            // if condition check
            if let Some(cond) = &step.if_cond {
                let val = ctx.resolve_var(cond).unwrap_or(Value::Null);
                if !is_truthy(&val) {
                    continue;
                }
            }

            // foreach block
            if let Some(foreach) = &step.foreach {
                let result = if foreach.parallel {
                    self.execute_foreach_parallel(foreach, &ctx, runner, max_iterations_hard).await?
                } else {
                    self.execute_foreach_serial(foreach, &ctx, runner, max_iterations_hard).await?
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

            // normal unit call
            if let Some(unit_name) = &step.unit {
                let step_input = Self::resolve_step_input(&ctx, &step.input)?;
                let (output, exit_code) = runner
                    .call(unit_name, Value::Object(step_input))
                    .await?;

                ctx = ctx.with_local_step(
                    step.name.clone(),
                    StepResult {
                        output,
                        exit_code,
                    },
                );
            }
        }

        self.build_return(&manifest.return_expr, &ctx)
    }

    async fn execute_foreach_serial(
        &self,
        foreach: &ForeachBlock,
        ctx: &ExecContext,
        runner: &UnitRunner,
        max_iterations_hard: u32,
    ) -> Result<Value> {
        // Resolve over expression to array
        let items = ctx
            .resolve_var(&foreach.over)
            .ok_or_else(|| anyhow!("foreach.over expression could not be resolved"))?
            .as_array()
            .ok_or_else(|| anyhow!("foreach.over must resolve to an array"))?
            .clone();

        // Empty array handling
        if items.is_empty() {
            return Ok(empty_aggregate(&foreach.aggregate));
        }

        // max_iterations check (hard limit from config)
        let max_iter = foreach.max_iterations.min(max_iterations_hard);
        if items.len() > max_iter as usize {
            anyhow::bail!(
                "Foreach attempted {} iterations (limit: {}). Hint: Increase max_iterations or batch process.",
                items.len(),
                max_iter
            );
        }

        // Snapshot: steps 是 Arc，Arc::clone() = O(1)
        let mut results: Vec<Value> = Vec::new();

        for (idx, item) in items.into_iter().enumerate() {
            // 创建子上下文 - O(1) 快照（Arc 克隆）
            let sub_ctx = ctx.fork_for_iteration(
                foreach.as_var.clone(),
                item.clone(),
                idx,
            );

            let mut step_ctx = sub_ctx;
            let mut success = true;
            let mut error_msg = None;

            for step in &foreach.flow {
                // if condition within foreach flow
                if let Some(cond) = &step.if_cond {
                    let val = step_ctx.resolve_var(cond).unwrap_or(Value::Null);
                    if !is_truthy(&val) {
                        continue;
                    }
                }

                let input = Self::resolve_step_input(&step_ctx, &step.input)?;

                if let Some(unit_name) = &step.unit {
                    match runner.call(unit_name, Value::Object(input)).await {
                        Ok((output, exit_code)) => {
                            step_ctx = step_ctx.with_local_step(
                                step.name.clone(),
                                StepResult {
                                    output,
                                    exit_code,
                                },
                            );
                        }
                        Err(e) => {
                            success = false;
                            error_msg = Some(e.to_string());
                            if foreach.on_error == ErrorStrategy::FailFast {
                                anyhow::bail!("Foreach iteration {} failed: {}", idx, e);
                            }
                            break;
                        }
                    }
                }
            }

            if success {
                // 即使成功也调用 resolve_aggregate_item，保持模板结构一致
                let aggregated = Self::resolve_aggregate_item(&foreach.aggregate, &step_ctx)?;
                results.push(aggregated);
            } else if foreach.on_error == ErrorStrategy::Continue {
                // continue 模式：先按模板解析，失败字段返回 null，然后合并 __error
                let mut aggregated = Self::resolve_aggregate_item(&foreach.aggregate, &step_ctx)?;
                if let Some(obj) = aggregated.as_object_mut() {
                    obj.insert("__error".to_string(), Value::String(error_msg.unwrap_or_default()));
                } else {
                    // 如果模板解析为非对象，创建包含 __error 的对象
                    let mut err_obj = serde_json::Map::new();
                    err_obj.insert("__error".to_string(), Value::String(error_msg.unwrap_or_default()));
                    aggregated = Value::Object(err_obj);
                }
                results.push(aggregated);
            }
        }

        self.apply_aggregate_mode(&foreach.aggregate, results)
    }

    async fn execute_foreach_parallel(
        &self,
        foreach: &ForeachBlock,
        ctx: &ExecContext,
        runner: &UnitRunner,
        max_iterations_hard: u32,
    ) -> Result<Value> {
        use futures::stream::{FuturesUnordered, StreamExt};
        use std::sync::atomic::{AtomicBool, Ordering};

        let items = ctx
            .resolve_var(&foreach.over)
            .ok_or_else(|| anyhow!("foreach.over expression could not be resolved"))?
            .as_array()
            .ok_or_else(|| anyhow!("foreach.over must resolve to an array"))?
            .clone();

        if items.is_empty() {
            return Ok(empty_aggregate(&foreach.aggregate));
        }

        let max_iter = foreach.max_iterations.min(max_iterations_hard);
        if items.len() > max_iter as usize {
            anyhow::bail!(
                "Foreach attempted {} iterations (limit: {}). Hint: Increase max_iterations or batch process.",
                items.len(),
                max_iter
            );
        }

        // Limit concurrency to avoid overwhelming the system (default: min(50, items.len()))
        // Use max(1) to prevent永久挂起 when COGTOME_MAX_CONCURRENT=0
        let max_concurrent = std::env::var("COGTOME_MAX_CONCURRENT")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(50)
            .min(items.len())
            .max(1);
        let concurrency_limiter = Arc::new(Semaphore::new(max_concurrent));

        let cancel_token = CancellationToken::new();
        let fail_fast_flag = Arc::new(AtomicBool::new(false));
        let first_error: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        let mut futures = FuturesUnordered::new();

        for (idx, item) in items.into_iter().enumerate() {
            let foreach_clone = foreach.clone();
            let ctx_clone = ctx.clone();
            let runner_clone = runner.clone();
            let cancel_clone = cancel_token.clone();
            let ff_flag = fail_fast_flag.clone();
            let ff_error = first_error.clone();
            let permit_clone = concurrency_limiter.clone();

            futures.push(async move {
                // Acquire permit before starting (limits concurrency)
                let _permit = permit_clone.acquire().await.ok();

                // Check if cancelled before starting
                if cancel_clone.is_cancelled() {
                    return None;
                }

                let sub_ctx = ctx_clone.fork_for_iteration(
                    foreach_clone.as_var.clone(),
                    item.clone(),
                    idx,
                );

                let mut step_ctx = sub_ctx;
                let mut success = true;
                let mut error_msg = None;

                for step in &foreach_clone.flow {
                    // Check cancellation before each unit call (for fail_fast)
                    if cancel_clone.is_cancelled() {
                        return None;
                    }

                    if let Some(cond) = &step.if_cond {
                        let val = step_ctx.resolve_var(cond).unwrap_or(Value::Null);
                        if !is_truthy(&val) {
                            continue;
                        }
                    }

                    let input = match Self::resolve_step_input(&step_ctx, &step.input) {
                        Ok(i) => i,
                        Err(e) => {
                            success = false;
                            error_msg = Some(e.to_string());
                            if foreach_clone.on_error == ErrorStrategy::FailFast {
                                ff_flag.store(true, Ordering::SeqCst);
                                let mut err = ff_error.lock().await;
                                if err.is_none() {
                                    *err = Some(e.to_string());
                                }
                                cancel_clone.cancel();
                            }
                            break;
                        }
                    };

                    if let Some(ref unit_name) = step.unit {
                        match runner_clone.call(unit_name, Value::Object(input)).await {
                            Ok((output, exit_code)) => {
                                step_ctx = step_ctx.with_local_step(
                                    step.name.clone(),
                                    StepResult { output, exit_code },
                                );
                            }
                            Err(e) => {
                                success = false;
                                error_msg = Some(e.to_string());

                                if foreach_clone.on_error == ErrorStrategy::FailFast {
                                    ff_flag.store(true, Ordering::SeqCst);
                                    let mut err = ff_error.lock().await;
                                    if err.is_none() {
                                        *err = Some(e.to_string());
                                    }
                                    cancel_clone.cancel();
                                    return None;
                                }
                                break;
                            }
                        }
                    }
                }

                if success {
                    match Self::resolve_aggregate_item(&foreach_clone.aggregate, &step_ctx) {
                        Ok(v) => Some(Some(v)),
                        Err(_) => Some(None),
                    }
                } else if foreach_clone.on_error == ErrorStrategy::Continue {
                    match Self::resolve_aggregate_item(&foreach_clone.aggregate, &step_ctx) {
                        Ok(mut agg) => {
                            if let Some(obj) = agg.as_object_mut() {
                                if let Some(msg) = error_msg {
                                    obj.insert("__error".to_string(), Value::String(msg));
                                }
                            }
                            Some(Some(agg))
                        }
                        Err(_) => Some(None),
                    }
                } else {
                    Some(None)
                }
            });
        }

        let mut results: Vec<Value> = Vec::new();

        while let Some(result) = futures.next().await {
            if let Some(Some(v)) = result {
                results.push(v);
            }
        }

        // If fail_fast was triggered, return the first error
        if fail_fast_flag.load(Ordering::SeqCst) {
            let err = first_error.lock().await;
            if let Some(e) = err.as_ref() {
                anyhow::bail!("Parallel foreach failed (fail_fast): {}", e);
            }
        }

        self.apply_aggregate_mode(&foreach.aggregate, results)
    }

    fn resolve_step_input(
        ctx: &ExecContext,
        input: &HashMap<String, String>,
    ) -> Result<Map<String, Value>> {
        let mut step_input = serde_json::Map::new();
        for (k, v) in input {
            let val = ctx.resolve_var(v).unwrap_or(Value::Null);
            step_input.insert(k.clone(), val);
        }
        Ok(step_input)
    }

    fn resolve_aggregate_item(
        aggregate: &AggregateBlock,
        ctx: &ExecContext,
    ) -> Result<Value> {
        if aggregate.map.is_empty() {
            return Ok(Value::Null);
        }

        let mut obj = serde_json::Map::new();
        for (k, v) in &aggregate.map {
            let val = ctx.resolve_var(v).unwrap_or(Value::Null);
            obj.insert(k.clone(), val);
        }
        Ok(Value::Object(obj))
    }

    fn apply_aggregate_mode(
        &self,
        aggregate: &AggregateBlock,
        results: Vec<Value>,
    ) -> Result<Value> {
        match aggregate.mode {
            AggregateMode::Array => Ok(Value::Array(results)),
            AggregateMode::Object => {
                let mut obj = serde_json::Map::new();
                for (i, v) in results.into_iter().enumerate() {
                    obj.insert(i.to_string(), v);
                }
                Ok(Value::Object(obj))
            }
            AggregateMode::Sum => {
                let total: f64 = results
                    .iter()
                    .filter_map(|v| v.as_f64())
                    .sum();
                Ok(serde_json::json!(total))
            }
            AggregateMode::Join => {
                let sep = aggregate
                    .join
                    .as_ref()
                    .map(|j| j.separator.as_str())
                    .unwrap_or("");
                let parts: Vec<String> = results
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                Ok(Value::String(parts.join(sep)))
            }
        }
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

fn empty_aggregate(aggregate: &AggregateBlock) -> Value {
    match aggregate.mode {
        AggregateMode::Array => serde_json::json!([]),
        AggregateMode::Object => serde_json::json!({}),
        AggregateMode::Sum => serde_json::json!(0),
        AggregateMode::Join => serde_json::json!(""),
    }
}

// ============================================================================
// Structure Executor
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct StructureManifest {
    #[allow(dead_code)]
    pub name: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub kind: String,
    pub motifs: Vec<MotifRef>,
    #[serde(default)]
    #[allow(dead_code)]
    pub input_schema: Option<Value>,
    #[serde(default)]
    #[allow(dead_code)]
    pub output_schema: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct MotifRef {
    pub name: String,
}

/// Validates input against input_schema if defined in manifest.
/// Returns Ok(()) if no schema or validation passes.
fn validate_structure_input(manifest: &StructureManifest, input: &Value) -> Result<()> {
    if let Some(ref schema) = manifest.input_schema {
        validate_input(input, schema)?;
    }
    Ok(())
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
        max_iterations_hard: u32,
    ) -> Result<Value> {
        // Validate input against input_schema before execution
        validate_structure_input(manifest, &input)?;

        let mut current = input;

        for motif_ref in &manifest.motifs {
            let motif_path = skills
                .find_motif(&motif_ref.name)
                .ok_or_else(|| anyhow!("Motif '{}' not found", motif_ref.name))?;

            let motif_manifest = YamlMotifEngine::load(&motif_path)?;
            let engine = YamlMotifEngine;
            current = engine.execute(&motif_manifest, current, runner, max_iterations_hard).await?;
        }

        Ok(current)
    }
}

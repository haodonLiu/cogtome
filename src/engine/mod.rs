pub mod graph;
pub mod mcp_bridge;
pub mod unit_runner;

#[allow(unused_imports)]
pub use graph::{Edge, Graph, GraphValidationError, Node, Position};
pub use unit_runner::{UnitConcurrency, UnitRunner};
pub use mcp_bridge::{McpBridgeInput, McpBridgeUnit};

use crate::context::{ExecContext, StepResult};
use crate::discovery::SkillsDir;
use crate::error::{CogtomeError, ErrorCode, ErrorLayer};
use crate::validation::validate_input;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::sync::Arc;
use tracing::{error, info, Instrument};

// ============================================================================
// Motif Manifest v2 (JSON)
// ============================================================================

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MotifManifestV2 {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required_units: Vec<String>,
    pub graph: Graph,
    #[serde(default)]
    pub input_schema: Option<Value>,
    #[serde(default)]
    pub output_schema: Option<Value>,
}

// ============================================================================
// Graph Motif Engine (v2 JSON)
// ============================================================================

#[derive(Clone)]
pub struct GraphMotifEngine;

impl GraphMotifEngine {
    pub fn load(path: &Path) -> Result<MotifManifestV2> {
        let content = std::fs::read_to_string(path).with_context(|| {
            format!("Failed to read motif manifest: {}", path.display())
        })?;
        let manifest: MotifManifestV2 = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse motif manifest: {}", path.display()))?;
        Ok(manifest)
    }

    pub async fn execute(
        &self,
        manifest: &MotifManifestV2,
        input: Value,
        runner: &UnitRunner,
        _max_iterations_hard: u32,
    ) -> Result<Value> {
        let span = tracing::info_span!(
            "motif",
            motif.name = %manifest.name,
            node_count = manifest.graph.nodes.len()
        );

        let start_time = std::time::Instant::now();

        async move {
            // Validate graph before execution
            manifest.graph.validate().map_err(|e| {
                anyhow::anyhow!("Graph validation failed: {}", e)
            })?;

            let mut ctx = ExecContext::new(input);
            let start_id = Self::find_start_node(&manifest.graph)?;

            let run_result = self.execute_node(&manifest.graph, &start_id, runner, &mut ctx).await;

            // Extract return values (only if execution succeeded)
            let result = match run_result {
                Ok(()) => Self::extract_return_output(&manifest.graph, &ctx)?,
                Err(e) => return Err(e),
            };

            info!(motif = %manifest.name, result_keys = result.as_object().map(|o| o.len()).unwrap_or(0), "motif completed");

            // Trace hook: log execution to ~/.cogtome/traces/
            self.emit_trace(manifest.name.as_str(), start_time, &ctx, &result);

            Ok(result)
        }
        .instrument(span)
        .await
    }

    fn format_time(d: std::time::Duration) -> String {
        let secs = d.as_secs();
        let hours = (secs % 86400) / 3600;
        let mins = (secs % 3600) / 60;
        let seconds = secs % 60;
        let days = secs / 86400;
        let mut year = 1970 + (days / 365) as i64;
        let mut remaining = (days % 365) as i64;
        let is_leap = |y: i64| y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
        let days_in_yr = |y: i64| if is_leap(y) { 366 } else { 365 };
        while remaining >= days_in_yr(year) {
            remaining -= days_in_yr(year);
            year += 1;
        }
        format!("{}d{}h:{:02}:{:02}", year, remaining, hours, mins)
    }

    fn format_date(d: std::time::Duration) -> String {
        let secs = d.as_secs();
        let days = secs / 86400;
        let mut year = 1970 + (days / 365) as i64;
        let mut remaining = (days % 365) as i64;
        let is_leap = |y: i64| y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
        let days_in_yr = |y: i64| if is_leap(y) { 366 } else { 365 };
        while remaining >= days_in_yr(year) {
            remaining -= days_in_yr(year);
            year += 1;
        }
        let days_in_month = |m: i64, y: i64| match m {
            1|3|5|7|8|10|12 => 31,
            4|6|9|11 => 30,
            2 => if is_leap(y) { 29 } else { 28 },
            _ => 30,
        };
        let mut month = 1;
        while remaining >= days_in_month(month, year) {
            remaining -= days_in_month(month, year);
            month += 1;
        }
        format!("{}-{:02}-{:02}", year, month, remaining + 1)
    }

    fn emit_trace(&self, skill_name: &str, start_time: std::time::Instant, ctx: &ExecContext, _result: &Value) {
        use std::io::Write;

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default();

        // Format: YYYY-MM-DD HH:MM:SS
        let timestamp = Self::format_time(now);
        // Format: YYYY-MM-DD (for filename)
        let date_str = Self::format_date(now);

        // Collect node traces from ExecContext.steps
        let node_traces: Vec<serde_json::Value> = ctx.steps
            .iter()
            .map(|(node_id, step)| {
                let ok = step.exit_code == 0;
                serde_json::json!({
                    "id": node_id,
                    "type": "unit",
                    "ok": ok,
                    "exit_code": step.exit_code,
                    "error": if ok { "" } else { "non-zero exit" }
                })
            })
            .collect();

        let trace_record = serde_json::json!({
            "trace_id": uuid::Uuid::new_v4().to_string(),
            "skill": skill_name,
            "started_at": timestamp,
            "duration_ms": duration_ms,
            "status": "success",
            "nodes": node_traces
        });

        // Write directly to ~/.cogtome/traces/<skill>/<date>.jsonl
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let trace_dir = std::path::PathBuf::from(home)
            .join(".cogtome")
            .join("traces")
            .join(skill_name);

        let file = trace_dir.join(format!("{}.jsonl", date_str));
        if let Some(parent) = file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&file) {
            let _ = writeln!(f, "{}", trace_record);
        }
    }

    fn find_start_node(graph: &Graph) -> Result<String> {
        for node in &graph.nodes {
            if matches!(node, Node::Start { .. }) {
                return Ok(node.id().to_string());
            }
        }
        anyhow::bail!("No start node found in graph")
    }

    async fn execute_node(
        &self,
        graph: &Graph,
        node_id: &str,
        runner: &UnitRunner,
        ctx: &mut ExecContext,
    ) -> Result<()> {
        let node = graph.find_node(node_id)
            .ok_or_else(|| anyhow::anyhow!("Node '{}' not found", node_id))?;

        match node {
            Node::Start { .. } => {
                let next = Self::find_next(graph, node_id, None)?;
                Box::pin(self.execute_node(graph, &next, runner, ctx)).await?;
            }

            Node::Unit { id, unit, input, on_error, .. } => {
                let resolved_input = Self::resolve_input(input, ctx)?;
                let result = runner.call(unit, resolved_input, None).await;

                match result {
                    Ok((output, _exit_code)) => {
                        Self::set_step_result(ctx, id.clone(), output, 0);
                        let next = Self::find_next(graph, node_id, None)?;
                        Box::pin(self.execute_node(graph, &next, runner, ctx)).await?;
                    }
                    Err(e) => {
                        match on_error {
                            Some(graph::OnErrorConfig { strategy: graph::ErrorStrategy::Continue, .. }) => {
                                Self::set_step_result(ctx, id.clone(), serde_json::json!({ "__error": e.to_string() }), -1);
                                let next = Self::find_next(graph, node_id, None)?;
                                Box::pin(self.execute_node(graph, &next, runner, ctx)).await?;
                            }
                            Some(graph::OnErrorConfig { strategy: graph::ErrorStrategy::Fallback, fallback_node: Some(fb) }) => {
                                Box::pin(self.execute_node(graph, fb, runner, ctx)).await?;
                            }
                            _ => return Err(e.into()),
                        }
                    }
                }
            }

            Node::If { id, condition, .. } => {
                let condition_result = Self::evaluate_condition(condition, ctx)?;
                let label = if condition_result { "true" } else { "false" };
                Self::set_step_result(ctx, id.clone(), serde_json::json!({ "condition": condition_result }), 0);
                let next = Self::find_next(graph, node_id, Some(label))?;
                Box::pin(self.execute_node(graph, &next, runner, ctx)).await?;
            }

            Node::Match { id, on, .. } => {
                let value = Self::evaluate_expression(on, ctx)?;
                let value_str = value.as_str().unwrap_or("").to_string();

                let edges = graph.outgoing_edges(node_id);
                let mut matched = false;
                for edge in edges {
                    if let Some(label) = &edge.label {
                        if label == &value_str || label == "default" {
                            Box::pin(self.execute_node(graph, &edge.target, runner, ctx)).await?;
                            matched = true;
                            break;
                        }
                    }
                }
                if !matched {
                    anyhow::bail!("Match node '{}' no branch matched value '{}'", id, value_str);
                }
                Self::set_step_result(ctx, id.clone(), value, 0);
            }

            Node::Foreach { id, over, as_var, max_iterations, subgraph, .. } => {
                let array_value = Self::evaluate_expression(over, ctx)?;
                let items = array_value.as_array()
                    .ok_or_else(|| anyhow::anyhow!("Foreach 'over' did not evaluate to array"))?;

                let limit = (*max_iterations).min(50) as usize;
                let items: Vec<_> = items.iter().take(limit).collect();
                let mut results = Vec::new();

                // Sequential execution
                for item in items {
                    let mut sub_ctx = ctx.clone();
                    sub_ctx.locals.insert(as_var.clone(), item.clone());

                    Box::pin(self.execute_node(subgraph, &Self::find_start_node(subgraph)?, runner, &mut sub_ctx)).await?;
                    results.push(Self::extract_return_output(subgraph, &sub_ctx)?);
                }

                Self::set_step_result(ctx, id.clone(), Value::Array(results), 0);
                let next = Self::find_next(graph, node_id, None)?;
                Box::pin(self.execute_node(graph, &next, runner, ctx)).await?;
            }

            Node::Fork { id, .. } => {
                // Sequential fork execution
                let edges = graph.outgoing_edges(node_id);
                for edge in edges {
                    Box::pin(self.execute_node(graph, &edge.target, runner, ctx)).await?;
                }

                let join_id = Self::find_join_point(graph, id)?;
                Box::pin(self.execute_node(graph, &join_id, runner, ctx)).await?;
            }

            Node::Join { id, .. } => {
                Self::set_step_result(ctx, id.clone(), serde_json::json!(null), 0);
                let next = Self::find_next(graph, node_id, None)?;
                if !next.is_empty() {
                    Box::pin(self.execute_node(graph, &next, runner, ctx)).await?;
                }
            }

            Node::Return { id, values, .. } => {
                let resolved: HashMap<String, Value> = values
                    .iter()
                    .map(|(k, v)| {
                        let val = Self::evaluate_expression(v, ctx).unwrap_or(Value::Null);
                        (k.clone(), val)
                    })
                    .collect();
                Self::set_step_result(ctx, id.clone(), Value::Object(resolved.into_iter().collect()), 0);
            }

            Node::MotifRef { id, motif, .. } => {
                Self::set_step_result(ctx, id.clone(), serde_json::json!({ "motif": motif }), 0);
            }

            Node::Gate { id, message, timeout, on_timeout, .. } => {
                let prompt = if message.is_empty() {
                    format!("Gate '{}': Continue? [y/N] ", id)
                } else {
                    format!("{} [y/N] ", message)
                };
                eprint!("{}", prompt);

                // Spawn blocking read on a dedicated thread to avoid blocking the async executor
                let read_future = tokio::task::spawn_blocking(move || {
                    use std::io::{self, BufRead};
                    let stdin = io::stdin();
                    let mut input = String::new();
                    match stdin.lock().read_line(&mut input) {
                        Ok(_) => input.trim().eq_ignore_ascii_case("y"),
                        Err(_) => false,
                    }
                });

                let confirmed = if *timeout > 0 {
                    match tokio::time::timeout(
                        tokio::time::Duration::from_secs(*timeout),
                        read_future
                    ).await {
                        Ok(Ok(result)) => result,
                        Ok(Err(_)) => false,
                        Err(_) => {
                            tracing::warn!(gate = %id, timeout = timeout, "Gate timed out awaiting user input");
                            false
                        }
                    }
                } else {
                    match read_future.await {
                        Ok(result) => result,
                        Err(_) => false,
                    }
                };

                if confirmed {
                    Self::set_step_result(ctx, id.clone(), serde_json::json!({ "approved": true }), 0);
                    let next = Self::find_next(graph, node_id, None)?;
                    Box::pin(self.execute_node(graph, &next, runner, ctx)).await?;
                } else {
                    Self::set_step_result(ctx, id.clone(), serde_json::json!({ "approved": false }), 0);
                    match on_timeout {
                        graph::GateTimeoutAction::Escalate => {
                            anyhow::bail!("Gate '{}' denied - escalation required", id);
                        }
                        _ => {
                            anyhow::bail!("Gate '{}' denied by user", id);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn set_step_result(ctx: &mut ExecContext, id: String, output: Value, exit_code: i32) {
        // Clone the current Arc, insert the new step, create new Arc
        let current = (*ctx.steps).clone();
        let mut new_steps: HashMap<String, StepResult> = current.into_iter().collect();
        new_steps.insert(id, StepResult { output, exit_code });
        ctx.steps = Arc::new(new_steps);
    }

    fn find_next(graph: &Graph, node_id: &str, label: Option<&str>) -> Result<String> {
        let edges: Vec<_> = graph.edges.iter()
            .filter(|e| e.source == node_id)
            .filter(|e| {
                if let Some(l) = label {
                    e.label.as_deref() == Some(l)
                } else {
                    true
                }
            })
            .collect();

        if edges.is_empty() {
            anyhow::bail!("No outgoing edge from '{}' with label '{:?}'", node_id, label);
        }
        if edges.len() > 1 && label.is_none() {
            anyhow::bail!("Multiple unlabeled outgoing edges from '{}'", node_id);
        }

        Ok(edges[0].target.clone())
    }

    fn find_join_point(graph: &Graph, fork_id: &str) -> Result<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        for edge in &graph.edges {
            if edge.source == fork_id {
                queue.push_back(edge.target.clone());
            }
        }

        let mut incoming_count: HashMap<String, usize> = HashMap::new();
        for edge in &graph.edges {
            *incoming_count.entry(edge.target.clone()).or_default() += 1;
        }

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            if incoming_count.get(&current).copied().unwrap_or(0) > 1 {
                if let Some(node) = graph.nodes.iter().find(|n| n.id() == current) {
                    if matches!(node, Node::Join { .. }) {
                        return Ok(current);
                    }
                }
            }

            for edge in &graph.edges {
                if edge.source == current {
                    queue.push_back(edge.target.clone());
                }
            }
        }

        anyhow::bail!("Fork '{}' has no explicit join point", fork_id)
    }

    fn resolve_input(input: &HashMap<String, String>, ctx: &ExecContext) -> Result<Value> {
        let mut resolved = serde_json::Map::new();
        for (key, expr) in input {
            let value = Self::evaluate_expression(expr, ctx)?;
            resolved.insert(key.clone(), value);
        }
        Ok(Value::Object(resolved))
    }

    fn evaluate_condition(condition: &str, ctx: &ExecContext) -> Result<bool> {
        let value = Self::evaluate_expression(condition, ctx)?;
        Ok(value.as_bool().unwrap_or(false))
    }

    fn evaluate_expression(expr: &str, ctx: &ExecContext) -> Result<Value> {
        Ok(ctx.resolve_var(expr).unwrap_or(Value::Null))
    }

    fn extract_return_output(graph: &Graph, ctx: &ExecContext) -> Result<Value> {
        for node in graph.nodes.iter().rev() {
            if let Node::Return { id, .. } = node {
                if let Some(step) = ctx.steps.get(id) {
                    return Ok(step.output.clone());
                }
            }
        }
        Ok(Value::Null)
    }
}

// ============================================================================
// Structure Manifest (JSON)
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MotifRef {
    pub name: String,
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
        let manifest: StructureManifest = serde_json::from_str(&content).with_context(|| {
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
                    .with_hint("Ensure the motif is defined in skills/motifs/<name>.json")
                })?;

                let motif_manifest = GraphMotifEngine::load(&motif_path)?;
                let engine = GraphMotifEngine;
                info!(structure = %manifest.name, motif = %motif_ref.name, format = "json");
                current = engine.execute(&motif_manifest, current, runner, max_iterations_hard).await?;
            }

            info!(structure = %manifest.name, "structure completed");
            Ok(current)
        }
        .instrument(span)
        .await
    }
}

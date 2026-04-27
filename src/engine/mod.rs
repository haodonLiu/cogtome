pub mod foreach;
pub mod graph;
pub mod motif_manifest;
pub mod unit_runner;

pub use foreach::{
    emit_step_end, emit_step_start, execute_foreach_parallel, execute_foreach_serial,
    execute_unit_with_error_handling, resolve_step_input,
};
#[allow(unused_imports)]
pub use graph::{Edge, Graph, GraphValidationError, Node, Position};
#[allow(unused_imports)]
pub use motif_manifest::{
    AggregateBlock, AggregateMode, BackoffStrategy, ErrorStrategy, FlowStep, ForeachBlock,
    JoinConfig, RetryConfig, StepErrorStrategy,
};
pub use unit_runner::{UnitConcurrency, UnitRunner};

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

        async move {
            // Validate graph before execution
            manifest.graph.validate().map_err(|e| {
                anyhow::anyhow!("Graph validation failed: {}", e)
            })?;

            let mut ctx = ExecContext::new(input);
            let start_id = Self::find_start_node(&manifest.graph)?;

            self.execute_node(&manifest.graph, &start_id, runner, &mut ctx).await?;

            // Extract return values
            let result = Self::extract_return_output(&manifest.graph, &ctx)?;
            info!(motif = %manifest.name, result_keys = result.as_object().map(|o| o.len()).unwrap_or(0), "motif completed");
            Ok(result)
        }
        .instrument(span)
        .await
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
        ctx.resolve_var(expr).ok_or_else(|| anyhow::anyhow!("Failed to evaluate: {}", expr))
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

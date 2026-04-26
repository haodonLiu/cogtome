use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// ============================================================================
// Motif Manifest (YAML)
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
    #[serde(default)]
    pub env_whitelist: Option<Vec<String>>,
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

// ============================================================================
// Structure Manifest
// ============================================================================

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct MotifRef {
    pub name: String,
}

// ============================================================================
// Default Functions
// ============================================================================

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

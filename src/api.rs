use crate::discovery::{extract_first_structure, ComplexInfo, SkillsDir};
use crate::engine::{StructureExecutor, UnitConcurrency, UnitRunner, YamlMotifEngine};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn start_server(port: u16, skills: SkillsDir, timeout: u64) -> anyhow::Result<()> {
    let state = AppState {
        skills,
        timeout,
        concurrency_config: HashMap::new(),
        running_tasks: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/complexes", get(list_complexes))
        .route("/complexes/:name", get(get_complex))
        .route("/run", post(run_execution))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("// HTTP API server listening on http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Clone)]
struct AppState {
    skills: SkillsDir,
    timeout: u64,
    concurrency_config: HashMap<String, UnitConcurrency>,
    #[allow(dead_code)]
    running_tasks: Arc<RwLock<HashMap<String, RunningTask>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunningTask {
    pub id: String,
    pub status: String,
}

async fn health_check() -> &'static str {
    "OK"
}

async fn list_complexes(
    State(state): State<AppState>,
) -> Json<Vec<ComplexInfo>> {
    Json(state.skills.discover_complexes().unwrap_or_default())
}

async fn get_complex(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let complex_path = state.skills.root.join(&name).join("SKILL.md");
    let content = std::fs::read_to_string(&complex_path)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let meta = crate::discovery::parse_skill_front_matter(&content)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "name": name,
        "description": meta.description,
        "structures": meta.structures
    })))
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum RunRequest {
    #[serde(rename = "complex")]
    Complex { name: String, input: serde_json::Value },
    #[serde(rename = "motif")]
    Motif { name: String, input: serde_json::Value },
    #[serde(rename = "structure")]
    Structure { name: String, input: serde_json::Value },
    #[serde(rename = "unit")]
    Unit { name: String, input: serde_json::Value },
}

#[derive(Debug, Serialize)]
pub struct RunResponse {
    pub result: serde_json::Value,
}

async fn run_execution(
    State(state): State<AppState>,
    Json(req): Json<RunRequest>,
) -> Result<Json<RunResponse>, StatusCode> {
    let runner = UnitRunner::new_with_config(
        state.skills.clone(),
        state.timeout,
        state.concurrency_config.clone(),
    );

    let result = match req {
        RunRequest::Complex { name, input } => {
            let complex_path = state.skills.root.join(&name);
            if !complex_path.exists() {
                return Err(StatusCode::NOT_FOUND);
            }

            let skill_md = std::fs::read_to_string(complex_path.join("SKILL.md"))
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let structure_name = extract_first_structure(&skill_md)
                .ok_or_else(|| StatusCode::NOT_FOUND)?;

            let path = state.skills.find_structure(&structure_name)
                .ok_or_else(|| StatusCode::NOT_FOUND)?;
            let manifest = StructureExecutor::load(&path)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let max_iter = state.skills.root.join("cogtome.toml");
            let max_hard = if max_iter.exists() {
                crate::config::CogtomeConfig::load(&max_iter)
                    .map(|c| c.runtime.max_iterations_hard)
                    .unwrap_or(500)
            } else {
                500
            };
            StructureExecutor::execute(&manifest, input, &state.skills, &runner, max_hard)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        RunRequest::Motif { name, input } => {
            let path = state.skills.find_motif(&name)
                .ok_or_else(|| StatusCode::NOT_FOUND)?;
            let manifest = YamlMotifEngine::load(&path)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let engine = YamlMotifEngine;
            let max_iter = state.skills.root.join("cogtome.toml");
            let max_hard = if max_iter.exists() {
                crate::config::CogtomeConfig::load(&max_iter)
                    .map(|c| c.runtime.max_iterations_hard)
                    .unwrap_or(500)
            } else {
                500
            };
            engine.execute(&manifest, input, &runner, max_hard)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        RunRequest::Structure { name, input } => {
            let path = state.skills.find_structure(&name)
                .ok_or_else(|| StatusCode::NOT_FOUND)?;
            let manifest = StructureExecutor::load(&path)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let max_iter = state.skills.root.join("cogtome.toml");
            let max_hard = if max_iter.exists() {
                crate::config::CogtomeConfig::load(&max_iter)
                    .map(|c| c.runtime.max_iterations_hard)
                    .unwrap_or(500)
            } else {
                500
            };
            StructureExecutor::execute(&manifest, input, &state.skills, &runner, max_hard)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        RunRequest::Unit { name, input } => {
            let (result, _exit_code) = runner.call(&name, input)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            result
        }
    };

    Ok(Json(RunResponse { result }))
}

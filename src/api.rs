use crate::discovery::{extract_first_structure, ComplexInfo, SkillsDir};
use crate::engine::{GraphMotifEngine, StructureExecutor, UnitConcurrency, UnitRunner};
use crate::error::CogtomeError;
use crate::metrics;
use crate::services::{
    list_motifs as service_list_motifs, list_structures as service_list_structures,
    list_units as service_list_units, read_structure as service_read_structure,
    save_unit as service_save_unit, validate_motif_by_name as service_validate_motif,
    validate_structure_by_name as service_validate_structure, write_structure as service_write_structure,
    MotifInfo, StructureInfo, UnitConfig, UnitInfo,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tower::ServiceExt;
use tower_http::services::ServeDir;
use tracing::{error, info};

fn validate_name(name: &str) -> Result<(), CogtomeError> {
    if name.is_empty() || name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(CogtomeError::new(
            crate::error::ErrorLayer::Runtime,
            crate::error::ErrorCode::EValidation,
            format!("Invalid name '{}': must be non-empty and contain no path traversal", name),
        ));
    }
    Ok(())
}

fn load_max_iterations_hard(skills_root: &std::path::Path) -> u32 {
    let config_path = skills_root.join("cogtome.toml");
    if config_path.exists() {
        if let Ok(config) = crate::config::CogtomeConfig::load(&config_path) {
            return config.runtime.max_iterations_hard;
        }
    }
    500
}

/// Resolve the path to the webui/dist directory.
fn resolve_webui_dir() -> Option<PathBuf> {
    // Check relative to the executable (production / standalone)
    let exe_dir = std::env::current_exe()
        .ok()?
        .parent()?
        .to_path_buf();
    let candidates = [
        exe_dir.join("webui/dist"),
        exe_dir.join("../webui/dist"),
        exe_dir.join("../../webui/dist"),
        // Development fallback: relative to the project root (CARGO_MANIFEST_DIR)
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("webui/dist"),
    ];
    for path in &candidates {
        if path.join("index.html").exists() {
            return Some(path.clone());
        }
    }
    None
}

pub async fn start_server(port: u16, skills: SkillsDir, timeout: u64) -> anyhow::Result<()> {
    let cancel_token = CancellationToken::new();
    start_server_with_shutdown(port, skills, timeout, cancel_token).await
}

pub async fn start_server_with_shutdown(
    port: u16,
    skills: SkillsDir,
    timeout: u64,
    cancel_token: CancellationToken,
) -> anyhow::Result<()> {
    let state = AppState {
        skills,
        timeout,
        concurrency_config: HashMap::new(),
        running_tasks: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .route("/complexes", get(list_complexes))
        .route("/complexes/:name", get(get_complex))
        .route("/run", post(run_execution))
        // Structure CRUD
        .route("/api/structures", get(list_structures_handler))
        .route("/api/structures/:name", get(get_structure_handler))
        .route("/api/structures/:name", put(put_structure_handler))
        .route("/api/structures/:name", delete(delete_structure_handler))
        // Motif (CRUD)
        .route("/api/motifs", get(list_motifs_handler))
        .route("/api/motifs/:name", get(get_motif_handler))
        .route("/api/motifs/:name", put(put_motif_handler))
        // Units
        .route("/api/units", get(list_units_handler))
        .route("/api/units/:name", get(get_unit_handler))
        .route("/api/units/:name", put(put_unit_handler))
        // Validation
        .route("/api/validate/:type/:name", post(validate_handler))
        .with_state(state);

    // Add CORS layer for API endpoints
    let app = app.layer(CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any));

    let app = if let Some(webui_dir) = resolve_webui_dir() {
        info!(dir = %webui_dir.display(), "Serving webui static files");
        app.nest_service("/", ServeDir::new(webui_dir).append_index_html_on_directories(true))
    } else {
        info!("Webui dist not found. Run 'cd webui && npm run build' to enable the web UI.");
        app
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(port = port, addr = %addr, "HTTP API server listening with /metrics endpoint");

    // Wait for server completion with graceful shutdown on cancel_token
    match axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            cancel_token.cancelled().await;
        })
        .await
    {
        Ok(_) => info!("HTTP server shut down gracefully"),
        Err(e) => error!(error = %e, "HTTP server error"),
    }

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

async fn metrics_handler() -> Json<metrics::MetricsSnapshot> {
    Json(metrics::snapshot())
}

async fn list_complexes(
    State(state): State<AppState>,
) -> Json<Vec<ComplexInfo>> {
    Json(state.skills.discover_complexes().unwrap_or_default())
}

async fn get_complex(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, CogtomeError> {
    validate_name(&name)?;
    let complex_path = state.skills.root.join(&name).join("SKILL.md");
    let content = std::fs::read_to_string(&complex_path)
        .map_err(|e| {
            error!(complex = %name, error = %e, "failed to read SKILL.md");
            CogtomeError::new(
                crate::error::ErrorLayer::Runtime,
                crate::error::ErrorCode::EComplexNotFound,
                format!("Failed to read SKILL.md for '{}': {}", name, e),
            )
        })?;

    let meta = crate::discovery::parse_skill_front_matter(&content)
        .map_err(|e| {
            error!(complex = %name, error = %e, "failed to parse SKILL.md front matter");
            CogtomeError::new(
                crate::error::ErrorLayer::Runtime,
                crate::error::ErrorCode::ERuntime,
                format!("Failed to parse SKILL.md front matter for '{}': {}", name, e),
            )
        })?;

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

/// API error response matching CogtomeError structure.
#[derive(Debug, Serialize)]
struct ApiErrorResponse {
    pub error: CogtomeError,
}

/// Wrap CogtomeError in an Axum response.
impl IntoResponse for CogtomeError {
    fn into_response(self) -> axum::response::Response {
        let status = match self.code {
            crate::error::ErrorCode::EComplexNotFound
            | crate::error::ErrorCode::EStructureNotFound
            | crate::error::ErrorCode::EMotifNotFound
            | crate::error::ErrorCode::EUnitNotFound => StatusCode::NOT_FOUND,
            crate::error::ErrorCode::EValidation
            | crate::error::ErrorCode::EMotifParse
            | crate::error::ErrorCode::EStructureParse => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(ApiErrorResponse { error: self });
        (status, body).into_response()
    }
}

async fn run_execution(
    State(state): State<AppState>,
    Json(req): Json<RunRequest>,
) -> Result<Json<RunResponse>, CogtomeError> {
    let runner = UnitRunner::new_with_config(
        state.skills.clone(),
        state.timeout,
        state.concurrency_config.clone(),
    );

    let result = match req {
        RunRequest::Complex { name, input } => {
            validate_name(&name)?;
            let complex_path = state.skills.root.join(&name);
            if !complex_path.exists() {
                return Err(CogtomeError::new(
                    crate::error::ErrorLayer::Runtime,
                    crate::error::ErrorCode::EComplexNotFound,
                    format!("Complex '{}' not found at {}", name, complex_path.display()),
                ));
            }

            let skill_md = std::fs::read_to_string(complex_path.join("SKILL.md"))
                .map_err(|e| {
                    error!(complex = %name, error = %e, "failed to read SKILL.md");
                    CogtomeError::new(
                        crate::error::ErrorLayer::Runtime,
                        crate::error::ErrorCode::ERuntime,
                        format!("Failed to read SKILL.md for '{}': {}", name, e),
                    )
                })?;
            let structure_name = extract_first_structure(&skill_md)
                .ok_or_else(|| {
                    CogtomeError::new(
                        crate::error::ErrorLayer::Runtime,
                        crate::error::ErrorCode::ERuntime,
                        format!("No structure found in Complex '{}'", name),
                    )
                })?;

            let path = state.skills.find_structure(&structure_name)
                .ok_or_else(|| {
                    CogtomeError::new(
                        crate::error::ErrorLayer::Runtime,
                        crate::error::ErrorCode::EStructureNotFound,
                        format!("Structure '{}' not found", structure_name),
                    )
                })?;
            let manifest = StructureExecutor::load(&path)
                .map_err(CogtomeError::from)?;
            let max_hard = load_max_iterations_hard(&state.skills.root);
            StructureExecutor::execute(&manifest, input, &state.skills, &runner, max_hard)
                .await
                .map_err(CogtomeError::from)?
        }
        RunRequest::Motif { name, input } => {
            let path = state.skills.find_motif(&name)
                .ok_or_else(|| {
                    CogtomeError::new(
                        crate::error::ErrorLayer::Motif,
                        crate::error::ErrorCode::EMotifNotFound,
                        format!("Motif '{}' not found", name),
                    )
                })?;
            let manifest = GraphMotifEngine::load(&path)
                .map_err(CogtomeError::from)?;
            let engine = GraphMotifEngine;
            let max_hard = load_max_iterations_hard(&state.skills.root);
            engine.execute(&manifest, input, &runner, max_hard)
                .await
                .map_err(CogtomeError::from)?
        }
        RunRequest::Structure { name, input } => {
            let path = state.skills.find_structure(&name)
                .ok_or_else(|| {
                    CogtomeError::new(
                        crate::error::ErrorLayer::Runtime,
                        crate::error::ErrorCode::EStructureNotFound,
                        format!("Structure '{}' not found", name),
                    )
                })?;
            let manifest = StructureExecutor::load(&path)
                .map_err(CogtomeError::from)?;
            let max_hard = load_max_iterations_hard(&state.skills.root);
            StructureExecutor::execute(&manifest, input, &state.skills, &runner, max_hard)
                .await
                .map_err(CogtomeError::from)?
        }
        RunRequest::Unit { name, input } => {
            let (result, _exit_code) = runner.call(&name, input, None)
                .await?;
            result
        }
    };

    Ok(Json(RunResponse { result }))
}

// ============================================================================
// Structure CRUD API
// ============================================================================

async fn list_structures_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<StructureInfo>>, CogtomeError> {
    let structures = service_list_structures(&state.skills)
        .map_err(|e| CogtomeError::layer_runtime().with_hint(format!("{}", e)))?;
    Ok(Json(structures))
}

async fn get_structure_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, CogtomeError> {
    validate_name(&name)?;

    let path = state.skills
        .find_structure(&name)
        .ok_or_else(|| {
            CogtomeError::new(
                crate::error::ErrorLayer::Runtime,
                crate::error::ErrorCode::EStructureNotFound,
                format!("Structure '{}' not found", name),
            )
        })?;

    let content = std::fs::read_to_string(&path)
        .map_err(|e| CogtomeError::layer_runtime().with_hint(format!("Failed to read: {}", e)))?;

    let json_value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| CogtomeError::layer_validation().with_hint(format!("Invalid JSON: {}", e)))?;

    Ok(Json(json_value))
}

#[derive(Debug, Deserialize)]
struct PutStructureRequest {
    name: String,
    motifs: Vec<crate::engine::MotifRef>,
    #[serde(default)]
    input_schema: Option<serde_json::Value>,
    #[serde(default)]
    output_schema: Option<serde_json::Value>,
}

async fn put_structure_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<PutStructureRequest>,
) -> Result<Json<serde_json::Value>, CogtomeError> {
    validate_name(&name)?;

    // Verify name matches path
    if req.name != name {
        return Err(CogtomeError::new(
            crate::error::ErrorLayer::Validation,
            crate::error::ErrorCode::EValidation,
            format!("Name mismatch: path is '{}' but body has '{}'", name, req.name),
        ));
    }

    let manifest = crate::engine::StructureManifest {
        name: req.name,
        kind: "structure".to_string(),
        motifs: req.motifs,
        input_schema: req.input_schema,
        output_schema: req.output_schema,
    };

    let dir_path = state.skills.root.join(&state.skills.structures_subdir).join(&name);
    let file_path = dir_path.join("manifest.json");

    service_write_structure(&file_path, &manifest)
        .map_err(|e| CogtomeError::layer_runtime().with_hint(format!("Failed to write: {}", e)))?;

    Ok(Json(serde_json::json!({ "message": "Structure saved", "path": file_path })))
}

async fn delete_structure_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, CogtomeError> {
    validate_name(&name)?;

    let dir_path = state
        .skills
        .root
        .join(&state.skills.structures_subdir)
        .join(&name);

    if !dir_path.exists() {
        return Err(CogtomeError::new(
            crate::error::ErrorLayer::Runtime,
            crate::error::ErrorCode::EStructureNotFound,
            format!("Structure '{}' not found", name),
        ));
    }

    std::fs::remove_dir_all(&dir_path).map_err(|e| {
        CogtomeError::layer_runtime().with_hint(format!("Failed to delete: {}", e))
    })?;

    Ok(Json(serde_json::json!({
        "message": "Structure deleted",
        "name": name
    })))
}

// ============================================================================
// Motif API (read-only)
// ============================================================================

async fn list_motifs_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<MotifInfo>>, CogtomeError> {
    let motifs = service_list_motifs(&state.skills)
        .map_err(|e| CogtomeError::layer_runtime().with_hint(format!("{}", e)))?;
    Ok(Json(motifs))
}

async fn get_motif_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<String, CogtomeError> {
    validate_name(&name)?;

    let path = state.skills
        .find_motif(&name)
        .ok_or_else(|| {
            CogtomeError::new(
                crate::error::ErrorLayer::Motif,
                crate::error::ErrorCode::EMotifNotFound,
                format!("Motif '{}' not found", name),
            )
        })?;

    let content = std::fs::read_to_string(&path)
        .map_err(|e| CogtomeError::layer_runtime().with_hint(format!("Failed to read: {}", e)))?;

    Ok(content)
}

async fn put_motif_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(content): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, CogtomeError> {
    validate_name(&name)?;

    let motif_path = state.skills.root.join(&state.skills.motifs_subdir).join(format!("{}.json", name));
    let json_str = serde_json::to_string_pretty(&content)
        .map_err(|e| CogtomeError::layer_runtime().with_hint(format!("Failed to serialize: {}", e)))?;

    if let Some(parent) = motif_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| CogtomeError::layer_runtime().with_hint(format!("Failed to create directory: {}", e)))?;
    }

    std::fs::write(&motif_path, json_str)
        .map_err(|e| CogtomeError::layer_runtime().with_hint(format!("Failed to write: {}", e)))?;

    Ok(Json(serde_json::json!({ "message": "Motif saved", "path": motif_path })))
}

// ============================================================================
// Units API
// ============================================================================

async fn list_units_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<UnitInfo>>, CogtomeError> {
    let units = service_list_units(&state.skills)
        .map_err(|e| CogtomeError::layer_runtime().with_hint(format!("{}", e)))?;
    Ok(Json(units))
}

async fn get_unit_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, CogtomeError> {
    validate_name(&name)?;
    Ok(Json(serde_json::json!({
        "name": name,
        "timeout": 30,
        "concurrency": 1,
        "description": ""
    })))
}

async fn put_unit_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, CogtomeError> {
    validate_name(&name)?;

    let config = UnitConfig {
        timeout: req.get("timeout").and_then(|v| v.as_u64()).map(|v| v as u32),
        concurrency: req.get("concurrency").and_then(|v| v.as_i64()).map(|v| v as i32),
        description: req.get("description").and_then(|v| v.as_str()).map(String::from),
    };

    let path = service_save_unit(&state.skills, &name, &config)
        .map_err(|e| CogtomeError::layer_runtime().with_hint(format!("{}", e)))?;

    Ok(Json(serde_json::json!({ "message": "Unit saved", "name": name, "path": path })))
}

// ============================================================================
// Validation API
// ============================================================================

async fn validate_handler(
    State(state): State<AppState>,
    Path((manifest_type, name)): Path<(String, String)>,
) -> Result<Json<crate::services::ValidationResult>, CogtomeError> {
    validate_name(&name)?;

    let result = match manifest_type.as_str() {
        "structure" => {
            service_validate_structure(&name, &state.skills)
                .map_err(|e| CogtomeError::layer_runtime().with_hint(format!("{}", e)))?
        }
        "motif" => {
            service_validate_motif(&name, &state.skills)
                .map_err(|e| CogtomeError::layer_runtime().with_hint(format!("{}", e)))?
        }
        _ => {
            return Err(CogtomeError::new(
                crate::error::ErrorLayer::Validation,
                crate::error::ErrorCode::EValidation,
                format!("Unknown type '{}': must be 'structure' or 'motif'", manifest_type),
            ));
        }
    };

    Ok(Json(result))
}

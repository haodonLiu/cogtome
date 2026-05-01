use crate::discovery::SkillsDir;
use crate::engine::protocol::parse_ndjson_output;
use crate::engine::sandbox::{SandboxRegistry, load_unit_manifest};
use crate::error::{CogtomeError, ErrorCode, ErrorLayer};
use crate::metrics;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Write as StdWrite;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::sync::{Mutex, Semaphore};
use tracing::{error, info, warn, Instrument};

// ============================================================================
// RAII guard for running task count
// ============================================================================

struct RunningTaskGuard;

impl Drop for RunningTaskGuard {
    fn drop(&mut self) {
        metrics::dec_running_tasks();
    }
}

// ============================================================================
// Unit Concurrency
// ============================================================================

#[derive(Debug, Clone)]
pub struct UnitConcurrency {
    /// Max concurrent executions. -1 means unlimited (u32::MAX permits).
    pub max_global: Option<i32>,
    #[allow(dead_code)]
    pub max_per_host: Option<i32>,
    pub resource_key: Option<String>,
}

impl UnitConcurrency {
    /// Returns the semaphore permits, or u32::MAX for unlimited (-1)
    pub fn permits(&self) -> usize {
        match self.max_global {
            Some(-1) => usize::MAX,
            Some(n) => n as usize,
            None => 1,
        }
    }
}

// ============================================================================
// Unit Runner
// ============================================================================

#[derive(Clone)]
pub struct UnitRunner {
    skills: SkillsDir,
    skills_root: std::path::PathBuf,
    timeout_secs: u64,
    concurrency_config: HashMap<String, UnitConcurrency>,
    sandbox_registry: SandboxRegistry,
    resource_semaphores: Arc<HashMap<String, Arc<Semaphore>>>,
    undeclared_semaphores: Arc<Mutex<HashMap<String, Arc<Semaphore>>>>,
}

impl UnitRunner {
    pub fn new_with_config(
        skills: SkillsDir,
        skills_root: std::path::PathBuf,
        timeout_secs: u64,
        concurrency_config: HashMap<String, UnitConcurrency>,
        sandbox_registry: SandboxRegistry,
    ) -> Self {
        // Build resource semaphores from config
        let mut resource_semaphores: HashMap<String, Arc<Semaphore>> = HashMap::new();
        for (_unit_name, config) in &concurrency_config {
            if let Some(ref key) = config.resource_key {
                // Use permits() which handles -1 (unlimited) and defaults to 1
                resource_semaphores.insert(key.clone(), Arc::new(Semaphore::new(config.permits())));
            }
        }

        Self {
            skills,
            skills_root,
            timeout_secs,
            concurrency_config,
            sandbox_registry,
            resource_semaphores: Arc::new(resource_semaphores),
            undeclared_semaphores: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn call(
        &self,
        name: &str,
        input: Value,
        env_whitelist: Option<&[String]>,
    ) -> Result<(Value, i32), CogtomeError> {
        let span = tracing::info_span!(
            "unit",
            unit.name = %name,
            unit.input_size = input.to_string().len()
        );

        async move {
            let _running = RunningTaskGuard;
            let start = Instant::now();

            // Acquire semaphore permit for rate limiting
            let sem = self.get_semaphore(name).await;
            let _permit = match sem.acquire().await {
                Ok(p) => p,
                Err(e) => {
                    let dur = start.elapsed().as_secs_f64();
                    metrics::record_unit_failure(name, "error", dur);
                    error!(unit = %name, error = %e, "semaphore acquire failed");
                    return Err(CogtomeError::new(ErrorLayer::Runtime, ErrorCode::ERuntime, format!("Semaphore error: {}", e)));
                }
            };

            let bin_path = match self.skills.find_unit(name) {
                Some(p) => p,
                None => {
                    let dur = start.elapsed().as_secs_f64();
                    metrics::record_unit_failure(name, "not_found", dur);
                    error!(unit = %name, "unit not found");
                    return Err(CogtomeError::new(ErrorLayer::Unit, ErrorCode::EUnitNotFound, format!("Unit '{}' not found", name))
                        .with_hint("Ensure the unit is installed in skills/units/<name>/bin/<name>"));
                }
            };

            // Create isolated temp directory for sandbox workspace
            let exec_id = uuid::Uuid::new_v4();
            let temp_dir = std::env::temp_dir().join(format!("cogtome-exec-{}", exec_id));

            if let Err(e) = std::fs::create_dir_all(&temp_dir) {
                let dur = start.elapsed().as_secs_f64();
                metrics::record_unit_failure(name, "error", dur);
                error!(unit = %name, path = %temp_dir.display(), error = %e, "failed to create temp directory");
                return Err(CogtomeError::new(ErrorLayer::Runtime, ErrorCode::ERuntime, format!("Failed to create temp directory: {}", e)));
            }

            // Load per-unit manifest (sandbox overrides, env whitelist, input_schema, etc.)
            let unit_manifest = load_unit_manifest(&self.skills_root, name);

            // P0-2: Input validation — check input against unit's JSON Schema before spawning.
            // Returns a structured Validation error instead of launching the process.
            if let Some(ref manifest) = unit_manifest {
                if let Some(ref schema) = manifest.input_schema {
                    if let Err(validation_err) = jsonschema::validate(schema, &input) {
                        let dur = start.elapsed().as_secs_f64();
                        metrics::record_unit_failure(name, "validation_error", dur);
                        error!(
                            unit = %name,
                            validation_error = %validation_err,
                            "unit input validation failed"
                        );
                        return Err(CogtomeError::new(
                            ErrorLayer::Validation,
                            ErrorCode::EValidation,
                            format!(
                                "Unit '{}' input validation failed: {}",
                                name,
                                validation_err
                            ),
                        )
                        .with_hint("Check the input passed to the unit matches the expected schema in the unit's manifest.yaml input_schema field"));
                    }
                }
            }

            // Resolve which sandbox backend to use
            let backend = self.sandbox_registry.resolve_for_unit(&unit_manifest);
            let sandbox_kind = if let Some(ref m) = unit_manifest {
                m.sandbox.unwrap_or_else(|| self.sandbox_registry.default_backend().kind())
            } else {
                self.sandbox_registry.default_backend().kind()
            };

            if !backend.is_available() {
                let dur = start.elapsed().as_secs_f64();
                metrics::record_unit_failure(name, "sandbox_unavailable", dur);
                let hint = backend.unavailable_hint();
                error!(unit = %name, sandbox = %sandbox_kind, "sandbox backend unavailable");
                return Err(CogtomeError::sandbox_unavailable(sandbox_kind, hint));
            }

            // Merge env whitelist: caller-supplied + manifest-supplied
            let manifest_whitelist = unit_manifest
                .as_ref()
                .map(|m| m.env_whitelist.clone())
                .unwrap_or_default();
            let mut combined_whitelist: Vec<String> = env_whitelist.unwrap_or(&[]).to_vec();
            combined_whitelist.extend(manifest_whitelist);

            // Build the Command using the sandbox backend
            let mut cmd = match backend.prepare_cmd(&bin_path, &temp_dir, unit_manifest.as_ref().unwrap_or(&Default::default())) {
                Ok(c) => c,
                Err(e) => {
                    let dur = start.elapsed().as_secs_f64();
                    metrics::record_unit_failure(name, "sandbox_error", dur);
                    error!(unit = %name, error = %e, "sandbox prepare_cmd failed");
                    return Err(CogtomeError::new(ErrorLayer::Runtime, ErrorCode::ERuntime, format!("Sandbox prepare failed: {}", e)));
                }
            };

            cmd.stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .current_dir(&temp_dir);

            // Always add COGTOME_UNIT_MODE
            cmd.env("COGTOME_UNIT_MODE", "1");
            // Add COGTOME_SANDBOX for introspection
            cmd.env("COGTOME_SANDBOX", sandbox_kind.to_string());

            // Add whitelisted env vars
            for var in &combined_whitelist {
                if let Ok(value) = std::env::var(var) {
                    cmd.env(var, value);
                }
            }

            // Spawn the child process using std::process::Command
            // (sandbox wrappers like bwrap/unshare are blocking std commands)
            let child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    let dur = start.elapsed().as_secs_f64();
                    metrics::record_unit_failure(name, "error", dur);
                    error!(unit = %name, path = %bin_path.display(), error = %e, "failed to spawn unit");
                    return Err(CogtomeError::new(ErrorLayer::Runtime, ErrorCode::ERuntime, format!("Failed to spawn unit '{}': {}", name, e)));
                }
            };

            // Wrap in Arc<Mutex> so we can take the child for kill-on-timeout
            let child_arc = Arc::new(Mutex::new(Some(child)));
            let child_for_kill = child_arc.clone();

            let input_bytes = input.to_string();
            let output = tokio::time::timeout(
                Duration::from_secs(self.timeout_secs),
                async {
                    // Take child from Arc<Mutex> and run blocking I/O on spawn_blocking thread
                    let child = {
                        let mut guard = child_arc.lock().await;
                        guard.take()
                    };

                    if let Some(child) = child {
                        let child = Arc::new(std::sync::Mutex::new(Some(child)));
                        let input_bytes = input_bytes.clone();

                        // Write stdin on blocking thread
                        let child_for_stdin = child.clone();
                        let _write_ok = tokio::task::spawn_blocking(move || {
                            let mut c = child_for_stdin.lock().unwrap();
                            if let Some(ref mut c) = *c {
                                if let Some(ref mut stdin) = c.stdin.take() {
                                    return stdin.write_all(input_bytes.as_bytes()).is_ok();
                                }
                            }
                            true
                        }).await.unwrap_or(true);

                        // Wait for output on blocking thread
                        let child_for_wait = child.clone();
                        tokio::task::spawn_blocking(move || {
                            let mut c = child_for_wait.lock().unwrap();
                            if let Some(child) = c.take() {
                                child.wait_with_output()
                            } else {
                                Err(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    "child already taken",
                                ))
                            }
                        }).await.unwrap_or_else(|_| {
                            Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "spawn_blocking task panicked",
                            ))
                        })
                    } else {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "child already taken",
                        ))
                    }
                },
            )
            .await;

            // Cleanup temp directory after execution
            let _ = fs::remove_dir_all(&temp_dir).await;

            let output = match output {
                Ok(Ok(output)) => output,
                Ok(Err(e)) => {
                    let dur = start.elapsed().as_secs_f64();
                    metrics::record_unit_failure(name, "io_error", dur);
                    error!(unit = %name, error = %e, "unit I/O error");
                    return Err(CogtomeError::new(
                        ErrorLayer::Unit,
                        ErrorCode::EUnitExec,
                        format!("Unit '{}' I/O error: {}", name, e),
                    ));
                }
                Err(_) => {
                    // Timeout - kill the child process
                    let mut guard = child_for_kill.lock().await;
                    if let Some(mut child) = guard.take() {
                        tokio::task::spawn_blocking(move || {
                            let _ = child.kill();
                        }).await.ok();
                    }
                    let dur = start.elapsed().as_secs_f64();
                    metrics::record_unit_failure(name, "timeout", dur);
                    warn!(unit = %name, timeout_secs = self.timeout_secs, "unit timed out");
                    return Err(CogtomeError::new(
                        ErrorLayer::Unit,
                        ErrorCode::EUnitTimeout,
                        format!("Unit '{}' timed out after {}s", name, self.timeout_secs),
                    )
                    .with_hint(format!(
                        "Increase timeout_secs for '{}' in cogtome.toml, or optimize the unit's execution time",
                        name
                    )));
                }
            };

            let exit_code = output.status.code().unwrap_or(-1);
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Handle non-zero exit codes with structured error mapping
            if exit_code != 0 {
                let dur = start.elapsed().as_secs_f64();
                let err = CogtomeError::from_exit_code(exit_code, name, &stderr);
                let status = match exit_code {
                    1 => "input_error",
                    2 => "retryable",
                    3 => "dep_unavailable",
                    _ => "nonzero",
                };
                metrics::record_unit_failure(name, status, dur);
                error!(
                    unit = %name,
                    exit_code = exit_code,
                    retryable = err.retryable,
                    "unit exited with error"
                );
                return Err(err);
            }

            // P0-1: Unit protocol — parse stdout using NDJSON-aware parser.
            // Accepts:
            //   - Single-line JSON output (legacy)
            //   - NDJSON with type-tagged lines (protocol v1)
            // In both cases only the {"type":"result",...} line is used.
            let result: Value = match parse_ndjson_output(&stdout) {
                Ok(v) => v,
                Err(e) => {
                    let dur = start.elapsed().as_secs_f64();
                    metrics::record_unit_failure(name, "parse_error", dur);
                    error!(
                        unit = %name,
                        parse_error = %e,
                        "unit output failed to parse via protocol rules"
                    );
                    return Err(CogtomeError::new(
                        ErrorLayer::Unit,
                        ErrorCode::EUnitExec,
                        format!(
                            "Unit '{}' output protocol violation: {} [{}] -- raw first line: {}",
                            name,
                            e.kind,
                            e.detail,
                            stdout.lines().next().unwrap_or("").chars().take(200).collect::<String>()
                        ),
                    ).with_hint("Units must print valid JSON to stdout. Prefer one JSON object per line, with logs going to stderr."))
                }
            };

            let dur = start.elapsed().as_secs_f64();
            metrics::record_unit_success(name, dur);
            info!(
                unit = %name,
                exit_code = exit_code,
                output_size = result.to_string().len(),
                "unit executed successfully"
            );

            return Ok((result, exit_code));
        }
        .instrument(span)
        .await
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

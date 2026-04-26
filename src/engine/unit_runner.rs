use crate::discovery::SkillsDir;
use crate::error::{CogtomeError, ErrorCode, ErrorLayer};
use crate::metrics;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
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
    pub fn new_with_config(
        skills: SkillsDir,
        timeout_secs: u64,
        concurrency_config: HashMap<String, UnitConcurrency>,
    ) -> Self {
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

            // Create isolated temp directory for security sandbox
            let exec_id = uuid::Uuid::new_v4();
            let temp_dir = std::env::temp_dir().join(format!("cogtome-exec-{}", exec_id));

            if let Err(e) = std::fs::create_dir_all(&temp_dir) {
                let dur = start.elapsed().as_secs_f64();
                metrics::record_unit_failure(name, "error", dur);
                error!(unit = %name, path = %temp_dir.display(), error = %e, "failed to create temp directory");
                return Err(CogtomeError::new(ErrorLayer::Runtime, ErrorCode::ERuntime, format!("Failed to create temp directory: {}", e)));
            }

            // Build Command with env whitelist - default is no inherited env vars
            let mut cmd = Command::new(&bin_path);
            cmd.stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .current_dir(&temp_dir);

            // Always add COGTOME_UNIT_MODE
            cmd.env("COGTOME_UNIT_MODE", "1");

            // Add whitelisted env vars if specified
            if let Some(whitelist) = env_whitelist {
                for var in whitelist {
                    if let Ok(value) = std::env::var(var) {
                        cmd.env(var, value);
                    }
                }
            }

            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    let dur = start.elapsed().as_secs_f64();
                    metrics::record_unit_failure(name, "error", dur);
                    error!(unit = %name, path = %bin_path.display(), error = %e, "failed to spawn unit");
                    return Err(CogtomeError::new(ErrorLayer::Runtime, ErrorCode::ERuntime, format!("Failed to spawn unit '{}': {}", name, e)));
                }
            };

            if let Some(mut stdin) = child.stdin.take() {
                if let Err(e) = stdin.write_all(input.to_string().as_bytes()).await {
                    let dur = start.elapsed().as_secs_f64();
                    metrics::record_unit_failure(name, "error", dur);
                    error!(unit = %name, error = %e, "stdin write failed");
                    return Err(CogtomeError::new(ErrorLayer::Unit, ErrorCode::EUnitExec, format!("stdin write failed: {}", e)));
                };
            }

            // Use Arc<Mutex<Option<Child>>> to allow taking child for kill on timeout
            let child_arc = Arc::new(Mutex::new(Some(child)));
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
                        let _ = child.kill().await;
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

            // Unit stdout/stderr protocol: first line is JSON result, rest is logs (ignored)
            let first_line = stdout.lines().next().unwrap_or("");
            let result: Value = match serde_json::from_str(first_line) {
                Ok(v) => v,
                Err(e) => {
                    let dur = start.elapsed().as_secs_f64();
                    metrics::record_unit_failure(name, "parse_error", dur);
                    error!(
                        unit = %name,
                        parse_error = %e,
                        raw_output = %first_line.chars().take(200).collect::<String>(),
                        "invalid JSON output from unit"
                    );
                    return Err(CogtomeError::new(
                        ErrorLayer::Unit,
                        ErrorCode::EUnitExec,
                        format!(
                            "Invalid JSON output from unit '{}': expected first line JSON, got: {}",
                            name,
                            first_line.chars().take(200).collect::<String>()
                        ),
                    )
                    .with_hint("Units must print exactly one JSON object as the first line of stdout"));
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

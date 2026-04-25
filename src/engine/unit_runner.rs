use crate::discovery::SkillsDir;
use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::{Mutex, Semaphore};

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
    ) -> Result<(Value, i32)> {
        // Acquire semaphore permit for rate limiting
        let sem = self.get_semaphore(name).await;
        let _permit = sem.acquire().await.map_err(|e| anyhow!("Semaphore error: {}", e))?;

        let bin_path = self
            .skills
            .find_unit(name)
            .with_context(|| format!("Unit '{}' not found", name))?;

        // Create isolated temp directory for security sandbox
        let exec_id = uuid::Uuid::new_v4();
        let temp_dir = std::env::temp_dir().join(format!("cogtome-exec-{}", exec_id));

        // Create temp directory
        std::fs::create_dir_all(&temp_dir)
            .with_context(|| format!("Failed to create temp directory: {}", temp_dir.display()))?;

        // Build Command with env whitelist - default is no inherited env vars
        let mut cmd = Command::new(&bin_path);
        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .current_dir(&temp_dir); // Restrict CWD to temp directory

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

        let mut child = cmd.spawn()
            .with_context(|| format!("Failed to spawn unit '{}'", name))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input.to_string().as_bytes()).await?;
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
            Ok(Err(e)) => anyhow::bail!("Unit '{}' I/O error: {}", name, e),
            Err(_) => {
                // Timeout - kill the child process
                let mut guard = child_for_kill.lock().await;
                if let Some(mut child) = guard.take() {
                    let _ = child.kill().await;
                }
                // Cleanup on timeout
                let _ = fs::remove_dir_all(&temp_dir).await;
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

        // Unit stdout/stderr protocol: first line is JSON result, rest is logs (ignored)
        let first_line = stdout.lines().next().unwrap_or("");
        let result: Value = serde_json::from_str(first_line).with_context(|| {
            format!(
                "Invalid JSON output from unit '{}': expected first line JSON, got: {}",
                name,
                first_line
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

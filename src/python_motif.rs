use anyhow::{anyhow, Result};
use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// JSON-RPC 2.0 response
#[derive(Debug, Deserialize)]
pub struct RpcResponse {
    #[serde(default)]
    pub jsonrpc: String,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<RpcError>,
    pub id: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub data: Option<Value>,
}

#[allow(dead_code)]
pub struct PythonMotifEngine {
    socket_path: PathBuf,
}

#[allow(dead_code)]
impl PythonMotifEngine {
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    pub async fn execute(&self, motif_name: &str, input: Value) -> Result<Value> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "execute",
            "params": {
                "motif_name": motif_name,
                "input": input
            },
            "id": 1
        });

        let response = self.send_rpc(request).await?;

        if let Some(err) = response.error {
            return Err(anyhow!("Python motif error ({}): {}", err.code, err.message));
        }

        response.result.ok_or_else(|| anyhow!("Python motif returned no result"))
    }

    async fn send_rpc(&self, request: Value) -> Result<RpcResponse> {
        let mut stream = UnixStream::connect(&self.socket_path).await?;

        // Send request as JSON line
        let request_str = serde_json::to_string(&request)?;
        stream.write_all(request_str.as_bytes()).await?;
        stream.write_all(b"\n").await?;

        // Read response line
        let mut response_str = String::new();
        stream.read_to_string(&mut response_str).await?;

        let response: RpcResponse = serde_json::from_str(&response_str)?;
        Ok(response)
    }
}

#[allow(dead_code)]
pub async fn start_python_server(script_path: &Path, socket_path: &Path) -> Result<tokio::process::Child> {
    // Ensure parent directory exists
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut child = tokio::process::Command::new("python3")
        .arg(script_path)
        .env("COGTOME_SOCKET_PATH", socket_path.to_string_lossy().as_ref())
        .spawn()?;

    // Wait for server to be ready (simple polling)
    for _ in 0..50 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        if socket_path.exists() {
            return Ok(child);
        }
    }

    // If we get here, server didn't start
    child.kill().await?;
    Err(anyhow!("Python motif server failed to start"))
}

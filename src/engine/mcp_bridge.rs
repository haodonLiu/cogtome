//! MCP Bridge Unit - Run MCP Servers as COGTOME Units
//!
//! Usage: `cogtome run mcp-bridge --input '{
//!   "server": "npx -y @modelcontextprotocol/server-filesystem /tmp",
//!   "tool": "read_text_file",
//!   "args": {"path": "/tmp/test.txt"}
//! }'`

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::Command as AsyncCommand;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpBridgeInput {
    /// MCP Server启动命令，如 "npx -y @modelcontextprotocol/server-filesystem /tmp"
    pub server: String,
    /// 要调用的工具名，如 "read_text_file"
    pub tool: String,
    /// 工具参数
    #[serde(default)]
    pub args: HashMap<String, Value>,
    /// 可选：初始化超时（秒）
    #[serde(default = "default_init_timeout")]
    pub init_timeout: u64,
    /// 可选：请求超时（秒）
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u64,
}

fn default_init_timeout() -> u64 {
    30
}

fn default_request_timeout() -> u64 {
    60
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpBridgeOutput {
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub tool: String,
}

pub struct McpBridgeUnit;

impl McpBridgeUnit {
    /// Execute an MCP tool via the bridge
    pub async fn execute(input: McpBridgeInput) -> Result<McpBridgeOutput> {
        let McpBridgeInput {
            server,
            tool,
            args,
            init_timeout,
            request_timeout,
        } = input;

        info!("MCP Bridge: starting server: {}", server);

        // Parse server command
        let parts: Vec<&str> = server.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow!("Empty server command"));
        }

        let (cmd, cmd_args) = if parts.len() == 1 {
            (parts[0], vec![])
        } else {
            (parts[0], parts[1..].to_vec())
        };

        debug!("MCP Bridge: cmd={}, args={:?}", cmd, cmd_args);

        // Start MCP server
        let mut child = AsyncCommand::new(cmd)
            .args(&cmd_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| anyhow!("Failed to spawn MCP server: {}", e))?;

        let mut stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let _stderr = child.stderr.take().unwrap();

        // Initialize MCP session
        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {
                    "name": "cogtome-mcp-bridge",
                    "version": "0.1.0"
                }
            }
        });

        stdin.write_all(format!("{}\n", init_request).as_bytes()).await.map_err(|e| anyhow!("Failed to write init: {}", e))?;
        stdin.flush().await.map_err(|e| anyhow!("Failed to flush: {}", e))?;

        // Read init response using tokio's buffered async reader
        let mut reader = tokio::io::BufReader::new(stdout);
        let mut line = String::new();
        let init_response = tokio::time::timeout(
            tokio::time::Duration::from_secs(init_timeout),
            reader.read_line(&mut line)
        ).await;

        if init_response.is_err() {
            return Err(anyhow!("MCP server init timeout"));
        }

        let init_result: Value = serde_json::from_str(&line)
            .map_err(|e| anyhow!("Failed to parse init response: {} - line: {}", e, line))?;

        debug!("MCP Bridge: init response: {:?}", init_result);

        // Send initialized notification
        let notif = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        });
        stdin.write_all(format!("{}\n", notif).as_bytes()).await.map_err(|e| anyhow!("Failed to send initialized: {}", e))?;
        stdin.flush().await.map_err(|e| anyhow!("Failed to flush: {}", e))?;

        // Call the tool
        let tool_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "1",
            "method": "tools/call",
            "params": {
                "name": tool,
                "arguments": args
            }
        });

        stdin.write_all(format!("{}\n", tool_request).as_bytes()).await.map_err(|e| anyhow!("Failed to write tool call: {}", e))?;
        stdin.flush().await.map_err(|e| anyhow!("Failed to flush: {}", e))?;

        // Read tool response
        let mut line = String::new();
        let tool_response = tokio::time::timeout(
            tokio::time::Duration::from_secs(request_timeout),
            reader.read_line(&mut line)
        ).await;

        if tool_response.is_err() {
            return Err(anyhow!("MCP tool request timeout"));
        }

        let response: Value = serde_json::from_str(&line)
            .map_err(|e| anyhow!("Failed to parse tool response: {}", e))?;

        debug!("MCP Bridge: tool response: {:?}", response);

        // Parse the response
        if let Some(result) = response.get("result") {
            if let Some(is_error) = result.get("isError").and_then(|v| v.as_bool()) {
                if is_error {
                    let error_content = result.get("content")
                        .and_then(|c| c.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|c| c.get("text"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("Unknown error")
                        .to_string();

                    return Ok(McpBridgeOutput {
                        success: false,
                        result: None,
                        error: Some(error_content),
                        tool,
                    });
                }
            }

            Ok(McpBridgeOutput {
                success: true,
                result: Some(result.clone()),
                error: None,
                tool,
            })
        } else if let Some(error) = response.get("error") {
            Ok(McpBridgeOutput {
                success: false,
                result: None,
                error: Some(error.to_string()),
                tool,
            })
        } else {
            Ok(McpBridgeOutput {
                success: false,
                result: Some(response),
                error: None,
                tool,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_filesystem_list_allowed() {
        let input = McpBridgeInput {
            server: "npx -y @modelcontextprotocol/server-filesystem /tmp".to_string(),
            tool: "list_allowed_directories".to_string(),
            args: HashMap::new(),
            init_timeout: 30,
            request_timeout: 30,
        };

        let result = McpBridgeUnit::execute(input).await;
        assert!(result.is_ok(), "Should succeed: {:?}", result);

        let output = result.unwrap();
        assert!(output.success, "Call should succeed: {:?}", output);
        assert!(output.result.is_some(), "Should have result");
    }

    #[tokio::test]
    async fn test_mcp_read_nonexistent_file() {
        let mut args = HashMap::new();
        args.insert("path".to_string(), Value::String("/tmp/nonexistent_file_12345.txt".to_string()));

        let input = McpBridgeInput {
            server: "npx -y @modelcontextprotocol/server-filesystem /tmp".to_string(),
            tool: "read_text_file".to_string(),
            args,
            init_timeout: 30,
            request_timeout: 30,
        };

        let result = McpBridgeUnit::execute(input).await;
        assert!(result.is_ok(), "Should succeed even with error: {:?}", result);

        let output = result.unwrap();
        assert!(!output.success, "Call should report error for missing file");
        assert!(output.error.is_some(), "Should have error message");
    }
}

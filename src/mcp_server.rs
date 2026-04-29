//! MCP Server - JSON-RPC 2.0 over stdio
//!
//! Implements the Model Context Protocol for COGTOME assemblies.
//! This allows COGTOME to be used as an MCP server by Claude/Cursor.

use crate::assembly::{Assembly, AssemblyRegistry, Tool};
use crate::discovery::SkillsDir;
use crate::engine::MotifManifestV2;
use crate::engine::UnitRunner;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, error, info};

/// MCP Protocol version (2024-11-05 is the current stable)
const PROTOCOL_VERSION: &str = "2024-11-05";

/// Server protocol state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServerState {
    Uninitialized,
    Initialized,
}

/// MCP Server capabilities
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<SamplingCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<RootsCapability>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolsCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourcesCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptsCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SamplingCapability {}

#[derive(Debug, Clone, Serialize)]
pub struct RootsCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Client capabilities (received during initialize)
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    #[serde(default)]
    pub tools: Option<Value>,
    #[serde(default)]
    pub resources: Option<Value>,
    #[serde(default)]
    pub prompts: Option<Value>,
    #[serde(default)]
    pub sampling: Option<Value>,
    #[serde(default)]
    pub roots: Option<Value>,
}

/// Initialize request params
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub protocol_version: Option<String>,
    pub capabilities: ClientCapabilities,
    pub client_info: ClientInfo,
}

#[derive(Debug, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// JSON-RPC 2.0 request
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP Server error codes
mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    pub const SERVER_NOT_INITIALIZED: i32 = -32002;
}

/// Tool call arguments
#[derive(Debug, Deserialize)]
pub struct ToolCallArgs {
    pub name: String,
    pub arguments: Option<Value>,
}

/// Resource item
#[derive(Debug, Clone, Serialize)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Prompt item
#[derive(Debug, Clone, Serialize)]
pub struct Prompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<PromptArgument>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

/// MCP Server state
pub struct McpServer {
    assemblies: Arc<AssemblyRegistry>,
    units_dir: PathBuf,
    timeout_secs: u64,
    state: ServerState,
    client_protocol_version: Option<String>,
    instructions: String,
}

impl McpServer {
    pub fn new(
        assemblies_dir: PathBuf,
        units_dir: PathBuf,
        timeout_secs: u64,
    ) -> Result<Self> {
        info!(
            assemblies = %assemblies_dir.display(),
            units = %units_dir.display(),
            "initializing MCP server"
        );

        let assemblies = Arc::new(AssemblyRegistry::new(assemblies_dir)?);

        Ok(Self {
            assemblies,
            units_dir,
            timeout_secs,
            state: ServerState::Uninitialized,
            client_protocol_version: None,
            instructions: String::new(),
        })
    }

    /// Get server capabilities
    fn get_capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
            resources: Some(ResourcesCapability {
                subscribe: Some(true),
                list_changed: Some(true),
            }),
            prompts: Some(PromptsCapability {
                list_changed: Some(true),
            }),
            sampling: Some(SamplingCapability {}),
            roots: Some(RootsCapability {
                list_changed: Some(true),
            }),
        }
    }

    /// Handle a JSON-RPC request
    pub fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone();
        let method = request.method.clone();

        // Check if client is trying to call methods before initialize
        // notifications/initialized is allowed before initialization completes
        let needs_initialization = method != "initialize" && method != "notifications/initialized";
        if self.state == ServerState::Uninitialized && needs_initialization {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: error_codes::SERVER_NOT_INITIALIZED,
                    message: "Server not initialized. Call initialize first.".to_string(),
                    data: None,
                }),
            };
        }

        match method.as_str() {
            "initialize" => self.handle_initialize(id, request.params),
            "tools/list" => self.handle_tools_list(id),
            "tools/call" => self.handle_tools_call(id, request.params),
            "resources/list" => self.handle_resources_list(id),
            "resources/read" => self.handle_resources_read(id, request.params),
            "prompts/list" => self.handle_prompts_list(id),
            "prompts/get" => self.handle_prompts_get(id, request.params),
            "ping" => self.handle_ping(id),
            "notifications/initialized" => {
                debug!("client initialized");
                self.state = ServerState::Initialized;
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Value::Null,
                    result: None,
                    error: None,
                }
            }
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: error_codes::METHOD_NOT_FOUND,
                    message: format!("Method not found: {}", method),
                    data: None,
                }),
            },
        }
    }

    fn handle_initialize(&mut self, id: Value, params: Value) -> JsonRpcResponse {
        let params: InitializeParams = match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => {
                error!(error = %e, "failed to parse initialize params");
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: error_codes::INVALID_PARAMS,
                        message: format!("Invalid params: {}", e),
                        data: None,
                    }),
                };
            }
        };

        debug!(
            client = %params.client_info.name,
            version = %params.client_info.version,
            protocol_version = ?params.protocol_version,
            "initialize request"
        );

        // Store client info for later use
        self.client_protocol_version = params.protocol_version;
        self.instructions = format!("COGTOME v{} - Agent Runtime Framework. Use tools to execute assemblies.",
            env!("CARGO_PKG_VERSION"));

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(serde_json::json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": self.get_capabilities(),
                "serverInfo": {
                    "name": "cogtome",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "instructions": self.instructions
            })),
            error: None,
        }
    }

    fn handle_tools_list(&self, id: Value) -> JsonRpcResponse {
        debug!(count = self.assemblies.list().len(), "tools/list request");

        let tools: Vec<Tool> = self.assemblies
            .all()
            .values()
            .map(Tool::from)
            .collect();

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(serde_json::json!({ "tools": tools })),
            error: None,
        }
    }

    fn handle_tools_call(&self, id: Value, params: Value) -> JsonRpcResponse {
        let args: ToolCallArgs = match serde_json::from_value(params) {
            Ok(a) => a,
            Err(e) => {
                error!(error = %e, "failed to parse tool call args");
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: error_codes::INVALID_PARAMS,
                        message: format!("Invalid params: {}", e),
                        data: None,
                    }),
                };
            }
        };

        debug!(tool = %args.name, "tools/call request");

        // Find the assembly
        let assembly = match self.assemblies.get(&args.name) {
            Some(a) => a,
            None => {
                error!(tool = %args.name, "assembly not found");
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: error_codes::INVALID_PARAMS,
                        message: format!("Assembly '{}' not found", args.name),
                        data: None,
                    }),
                };
            }
        };

        // Execute the workflow synchronously
        let input = args.arguments.unwrap_or(serde_json::json!({}));
        let result = self.execute_workflow_sync(assembly, input);

        match result {
            Ok(output) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&output).unwrap_or_else(|_| output.to_string())
                    }],
                    "isError": false
                })),
                error: None,
            },
            Err(e) => {
                error!(error = %e, "assembly execution failed");
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Error: {}", e)
                        }],
                        "isError": true
                    })),
                    error: None,
                }
            }
        }
    }

    fn handle_resources_list(&self, id: Value) -> JsonRpcResponse {
        debug!("resources/list request");

        let mut resources = Vec::new();

        // Expose assemblies as resources
        for (name, assembly) in self.assemblies.all() {
            resources.push(Resource {
                uri: format!("cogtome://assembly/{}", name),
                name: name.clone(),
                description: Some(assembly.manifest.description.clone()),
                mime_type: Some("application/json".to_string()),
            });
        }

        // Expose units as resources
        let units_path = std::env::current_dir().unwrap_or_default().join("units");
        if let Ok(entries) = std::fs::read_dir(&units_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() { continue; }
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                let unit_json_path = path.join("unit.json");
                let description = std::fs::read_to_string(&unit_json_path)
                    .ok()
                    .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
                    .and_then(|v| v.get("description")?.as_str().map(String::from));

                resources.push(Resource {
                    uri: format!("cogtome://unit/{}", name),
                    name: name.to_string(),
                    description,
                    mime_type: Some("application/json".to_string()),
                });
            }
        }

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(serde_json::json!({
                "resources": resources,
                "cursor": Value::Null
            })),
            error: None,
        }
    }

    fn handle_resources_read(&self, id: Value, params: Value) -> JsonRpcResponse {
        let uri = params.get("uri").and_then(|v| v.as_str()).unwrap_or("");

        debug!(uri = %uri, "resources/read request");

        // Parse resource URI
        let content = if uri.starts_with("cogtome://assembly/") {
            let name = uri.trim_start_matches("cogtome://assembly/");
            self.assemblies.get(name)
                .map(|a| {
                    serde_json::json!({
                        "contents": [{
                            "uri": uri,
                            "mime_type": "application/json",
                            "text": a.workflow_content().unwrap_or_default()
                        }]
                    })
                })
        } else if uri.starts_with("cogtome://unit/") {
            let name = uri.trim_start_matches("cogtome://unit/");
            let units_path = std::env::current_dir().unwrap_or_default().join("units");
            let unit_json_path = units_path.join(name).join("unit.json");
            std::fs::read_to_string(&unit_json_path)
                .ok()
                .map(|text| {
                    serde_json::json!({
                        "contents": [{
                            "uri": uri,
                            "mime_type": "application/json",
                            "text": text
                        }]
                    })
                })
        } else {
            None
        };

        match content {
            Some(c) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(c),
                error: None,
            },
            None => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: error_codes::INVALID_PARAMS,
                    message: format!("Resource not found: {}", uri),
                    data: None,
                }),
            },
        }
    }

    fn handle_prompts_list(&self, id: Value) -> JsonRpcResponse {
        debug!("prompts/list request");

        let prompts = vec![
            Prompt {
                name: "run-assembly".to_string(),
                description: Some("Execute an assembly with input".to_string()),
                arguments: vec![
                    PromptArgument {
                        name: "assembly".to_string(),
                        description: Some("Assembly name to execute".to_string()),
                        required: true,
                    },
                    PromptArgument {
                        name: "input".to_string(),
                        description: Some("JSON input for the assembly".to_string()),
                        required: true,
                    },
                ],
            },
            Prompt {
                name: "discover".to_string(),
                description: Some("List all available assemblies and units".to_string()),
                arguments: vec![],
            },
        ];

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(serde_json::json!({ "prompts": prompts })),
            error: None,
        }
    }

    fn handle_prompts_get(&self, id: Value, params: Value) -> JsonRpcResponse {
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");

        debug!(name = %name, "prompts/get request");

        match name {
            "run-assembly" => {
                let assembly = params.get("arguments")
                    .and_then(|a| a.get("assembly"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let input = params.get("arguments")
                    .and_then(|a| a.get("input"))
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "{}".to_string());

                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(serde_json::json!({
                        "messages": [{
                            "role": "user",
                            "content": {
                                "type": "text",
                                "text": format!(
                                    "Use the `tools/call` method to execute assembly '{}' with input: {}",
                                    assembly, input
                                )
                            }
                        }]
                    })),
                    error: None,
                }
            }
            "discover" => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(serde_json::json!({
                    "messages": [{
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": "Use `tools/list` to discover all available assemblies and their capabilities."
                        }
                    }]
                })),
                error: None,
            },
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: error_codes::METHOD_NOT_FOUND,
                    message: format!("Prompt not found: {}", name),
                    data: None,
                }),
            },
        }
    }

    fn handle_ping(&self, id: Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(serde_json::json!({})),
            error: None,
        }
    }

    /// Execute workflow synchronously using a dedicated async runtime in a background thread
    fn execute_workflow_sync(&self, assembly: &Assembly, input: Value) -> Result<Value> {
        let workflow_content = assembly.workflow_content()?;
        let manifest: MotifManifestV2 = serde_json::from_str(&workflow_content)
            .with_context(|| format!("Failed to parse workflow for '{}'", assembly.name()))?;

        info!(
            assembly = %assembly.name(),
            nodes = manifest.graph.nodes.len(),
            "executing workflow"
        );

        manifest.graph.validate().map_err(|e| anyhow::anyhow!("Graph validation failed: {}", e))?;

        let units_base = std::env::current_dir()?.join("units");

        let result = std::thread::scope(|s| {
            s.spawn(|| {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create tokio runtime");

                rt.block_on(async {
                    self.execute_workflow_async(manifest, input, &units_base).await
                })
            }).join().expect("Thread panicked")
        })?;

        Ok(result)
    }

    async fn execute_workflow_async(
        &self,
        manifest: MotifManifestV2,
        input: Value,
        units_base: &std::path::Path,
    ) -> Result<Value> {
        use crate::engine::GraphMotifEngine;

        let runner = UnitRunner::new_with_config(
            SkillsDir::with_subdirs(
                units_base.to_path_buf(),
                PathBuf::from("."),
                PathBuf::from("."),
                PathBuf::from("."),
            ),
            30,
            HashMap::new(),
        );

        let engine = GraphMotifEngine;
        engine.execute(&manifest, input, &runner, 500).await
    }
}

/// Run the MCP server synchronously
pub fn run_server(assemblies_dir: PathBuf, units_dir: PathBuf, timeout_secs: u64) -> Result<()> {
    let mut server = McpServer::new(assemblies_dir, units_dir, timeout_secs)?;

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    let reader = std::io::BufReader::new(stdin);
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                error!(error = %e, "failed to read line");
                continue;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        debug!(line = %line, "received request");

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Value::Null,
                    result: None,
                    error: Some(JsonRpcError {
                        code: error_codes::PARSE_ERROR,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                writeln!(stdout, "{}", serde_json::to_string(&response).unwrap()).ok();
                stdout.flush().ok();
                continue;
            }
        };

        let response = server.handle_request(request);

        if response.id != Value::Null {
            let response_json = serde_json::to_string(&response)?;
            writeln!(stdout, "{}", response_json).map_err(|e| anyhow!("Failed to write response: {}", e))?;
            stdout.flush().map_err(|e| anyhow!("Failed to flush stdout: {}", e))?;
        }
    }

    Ok(())
}

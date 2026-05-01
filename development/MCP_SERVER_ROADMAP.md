# MCP Server SDK Migration Analysis

> **Status**: Research complete — migration deferred, custom implementation maintained

## P1-1: MCP Server SDK Assessment

### Research Summary

The MCP (Model Context Protocol) is a JSON-RPC 2.0 protocol. The current COGTOME MCP Server (`src/mcp_server.rs`) is a **hand-rolled implementation** over stdio transport, supporting:

- `initialize`, `tools/list`, `tools/call`, `resources/*`, `prompts/*`, `ping`
- Protocol version `2024-11-05`
- Session pooling via MCP Bridge (`src/engine/mcp_bridge.rs`)

### Official/Community SDK Options

| Crate | Version | Rust Version | Status |
|-------|---------|--------------|--------|
| `rust-mcp-sdk` | 0.9.0 | 1.80+ | Active, full-featured |
| `mcp-sdk-rs` | 0.3.4 | unknown | Less active |
| `async-mcp` | 0.1.3 | — | Minimal |
| `rust-mcp-core` | 0.1.0 | — | Config-driven wrapper |

The most mature option is **`rust-mcp-sdk`** (rust-mcp-stack org), which supports:
- Stdio transport ✅
- **Streamable HTTP** ✅ (latest MCP protocol)
- SSE, macros, auth, hyper server
- Type-safe MCP schema objects

### Migration Assessment

**Benefits of migrating to `rust-mcp-sdk`:**
1. Protocol correctness — official schema validation
2. Streamable HTTP support (required for Claude Desktop with HTTP mode)
3. Auto-generated `inputSchema` from tool definitions
4. Ongoing protocol updates without manual maintenance

**Drawbacks/Costs:**
1. **Rust 1.80 minimum** — COGTOME currently supports Rust 1.70+ (`Cargo.toml` edition 2021)
2. **Breaking API changes** — current custom implementation is stable; SDK is at 0.x
3. **Limited custom logic** — COGTOME's `execute_workflow_async` + `McpBridge` integration requires adapter layer regardless
4. **Scope creep risk** — the existing implementation works; SDK benefit is mainly protocol correctness

### Recommendation

**Defer migration to Phase 3 or Phase 4**, revisit when:
1. MCP protocol specification stabilizes (currently at 2024-11-05, updates frequent)
2. `rust-mcp-sdk` reaches 1.0 (currently at 0.9.0)
3. COGTOME is ready to bump minimum Rust to 1.80

### `tools/list` inputSchema Auto-Generation

Currently in COGTOME, the `Tool::inputSchema` is derived from `AssemblyManifest.input_schema` (from `manifest.json`), which is already schema-free JSON. The existing implementation correctly exposes this as the MCP `inputSchema`.

If migrating to `rust-mcp-sdk`, the `#[derive(MCPTool)]` macro would auto-generate the schema from struct fields — this would require wrapping COGTOME's dynamic assembly manifests in typed structs, adding complexity without benefit.

### Streamable HTTP Support

The current stdio transport works for Claude Code/Cursor. Streamable HTTP (MCP's newer transport) would require:
1. Switching from stdin/stdout to HTTP server
2. Adding session management for HTTP connections
3. Integration with the existing `GraphMotifEngine::execute` async runtime

This is architecturally significant and best done as a separate feature branch.

## Current Architecture (P1 Status: ✅ Maintained, not migrated)

```
MCP Client (Claude Code)
    │
    │ JSON-RPC 2.0 (stdio)
    ▼
cogtome mcp-server
    │
    ├── AssemblyRegistry (loads assemblies/)
    │       └── manifest.json → Tool.inputSchema
    │
    └── GraphMotifEngine.execute()
            │
            └── UnitRunner (forks subprocess)
```

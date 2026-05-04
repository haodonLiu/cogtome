# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

先思考再编码 / 简洁优先 / 手术刀式修改 / 目标驱动执行

## Build and Run

```bash
cargo build --release                              # Build release binary
cargo test                                         # Run all tests
cargo test context::expression::tests               # Run expression engine tests
./target/release/cogtome discover                  # Scan all Structures
./target/release/cogtome run <structure> --input '<json>'   # Execute Structure
./target/release/cogtome unit run <name> --input '<json>' # Run Unit directly
./target/release/cogtome motif run <name> --input '<json>' # Run Motif (JSON v2)
./target/release/cogtome structure run <name> --input '<json>' # Run Structure
./target/release/cogtome serve --port 3334         # Start HTTP API server
./target/release/cogtome mcp-server --assemblies ./assemblies --units ./units  # MCP server (stdio)
```

**Environment variables:**
- `COGTOME_SKILLS_DIR` — skills root (default: `$(cargo manifest_dir)/skills`)
- `COGTOME_TIMEOUT` — unit timeout in seconds (default: 30)
- `COGTOME_MAX_CONCURRENT` — max parallel foreach iterations (default: 50, min: 1)

## Architecture

COGTOME is a micro OS for AI Agents with three execution layers:

```
Agent -> Structure -> Motif -> Unit
```

- **Structure**: Chains motifs sequentially. Two kinds: skills/ (with SKILL.md) and assemblies/ (with manifest.json)
- **Motif**: DAG orchestration via `MotifManifestV2` (JSON with `graph{nodes, edges}`)
- **Unit**: Atomic executor — fork+exec CLI, stdin/stdout JSON

**Core discipline**: Units never call each other. All cross-layer calls go through Runtime IPC.

### MCP Server Mode

COGTOME runs as an MCP server (JSON-RPC 2.0 over stdio):

```bash
./target/release/cogtome mcp-server --assemblies ./assemblies --units ./units
```

- `tools/list` — returns all discovered assemblies as MCP tools
- `tools/call` — executes the assembly's `workflow.json` (a `MotifManifestV2` DAG) via `GraphMotifEngine`

### v2 Manifest Format (JSON DAG)

Motifs use JSON with a directed graph model:
- `graph.nodes`: start, unit, if, match, foreach, fork, join, return, motifRef
- `graph.edges`: source/target with optional labels (for if/match branches)
- Validation rules: exactly one start, at least one return, no cycles, fork/join pairing

## Source Modules (src/)

| File | Responsibility |
|------|----------------|
| `main.rs` | CLI entry via clap. Routes to Unit/Motif/Structure/Run/Discover/Serve/McpServer/Pack/Install/Reload |
| `assembly.rs` | `AssemblyRegistry` — discovers assemblies from `assemblies/` via `manifest.json` |
| `mcp_server.rs` | MCP server — JSON-RPC 2.0 stdio, `tools/list`, `tools/call` |
| `engine/mod.rs` | `GraphMotifEngine` (v2 JSON DAG executor), `StructureExecutor`, `MotifManifestV2` |
| `engine/graph.rs` | `Graph` validation — cycles, start/return counts, fork/join pairing |
| `engine/unit_runner.rs` | `UnitRunner` — tokio process execution with semaphore rate limiting, timeout |
| `context/variables.rs` | `ExecContext` — variable resolution `${...}`, step storage with Arc-snapshot |
| `context/expression.rs` | Expression engine — filter, map, conditions, ternary, array indexing |
| `discovery.rs` | `SkillsDir` — discovers structures from skills/ and assemblies/ directories |
| `config.rs` | `CogtomeConfig` — `cogtome.toml` parsing, concurrency (supports `max_global = -1` = unlimited) |
| `api.rs` | HTTP API (Axum) — `/health`, `/structures`, `/api/motifs`, `/run` |
| `pack.rs` | `.cogtome` archive pack/install (tar+gzip) |
| `validation.rs` | JSON Schema validation via `jsonschema` crate |

**Unit contract**: stdin/stdout JSON. Exit codes: 0=success, 1=input error, 2=retryable, 3=dependency unavailable.

## Directory Structure

```
skills/                          # Skills root (COGTOME_SKILLS_DIR)
├── units/<name>/bin/<name>     # Executable Unit
├── motifs/<name>.json          # JSON v2 Motif (DAG format)
├── structures/<name>/manifest.json  # Structure manifest
└── <name>/SKILL.md             # Structure facade

assemblies/                      # MCP Server assemblies
└── <assembly>/
    ├── manifest.json           # name, description, units[], workflow
    └── workflow.json           # MotifManifestV2 (the DAG to execute)

units/                           # Standalone units
└── <name>/
    ├── unit.json               # name, entry, input_schema, output_schema
    └── bin/<name>              # Executable
```

## Config File (cogtome.toml)

```toml
[runtime]
max_iterations = 50        # default per-foreach limit
max_iterations_hard = 500  # absolute hard limit

[units.defaults]
timeout_secs = 30

[units.concurrency.my-unit]
max_global = -1            # -1 = unlimited (default: 1 = serialized)
resource_key = "api_quota" # share semaphore across units with same key
```

## Key Implementation Notes

### Variable Resolution
- `${params.x}` — user input parameters
- `${steps.name.output.field}` — step outputs (Arc-cloned, O(1) snapshot)
- `${env.VAR}` — environment variables
- Array index: `${arr[0]}`, `${arr[-1]}` (negative from end)
- Length: `${arr.length}`

### Concurrency Control
- `-1` in `max_global` means unlimited (`usize::MAX` permits)
- Default is `1` (serialized per unit name)
- `resource_key` groups units sharing a semaphore

### Graph Node Types (v2)
- `start` — entry point (exactly one required)
- `unit` — execute a Unit with `${...}` input resolution
- `if` — branches on condition (edges labeled: true/false)
- `match` — multi-way branch on value
- `foreach` — loop with inline subgraph
- `fork` / `join` — parallel execution barrier
- `return` — output values (at least one required)
- `motifRef` — reference another motif

### Snapshot Semantics
Foreach iterations clone `Arc<HashMap>` for steps — O(1) snapshot, isolation per iteration.

### Error Handling
- Exit codes: 0=success, 1=input error, 2=retryable, 3=dependency unavailable
- Timeout kill uses `Arc<Mutex<Option<Child>>>` pattern
- On error strategy: `fail_fast` (default) or `continue` (collects errors in `__error` field)
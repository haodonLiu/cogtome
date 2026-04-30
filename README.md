<img src="cover.jpg" width="400" alt="COGTOME" />

> English | [中文版本](README_CN.md)

# COGTOME

> **Agent's execution layer constraint — reduce hallucinations, improve reliability.**

> COGTOME gives AI Agents a tested, reusable execution playbook. The Agent decides *what* to do; COGTOME ensures the execution follows the correct DAG, handles errors, and maintains state.

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## Table of Contents

1. [What is COGTOME](#what-is-cogtome)
2. [Key Features](#key-features)
3. [Architecture](#architecture)
4. [Sandbox Isolation Strategy](#sandbox-isolation-strategy)
5. [Quick Start](#quick-start)
6. [Project Structure](#project-structure)
7. [CLI Reference](#cli-reference)
8. [Comparison](#comparison)
9. [Design Principles](#design-principles)
10. [Phase Status](#phase-status)

---

## What is COGTOME

COGTOME is a **runtime that executes declarative workflows as Agent tools** — process isolation, DAG orchestration, state propagation, and observable execution traces.

### The Problem

Agents know *what* to do but often fail *how* to do it:
- Tool calls in wrong order
- Parameters passed incorrectly
- Errors handled incompletely
- Multi-step state lost mid-execution

### The Solution

COGTOME provides a tested execution blueprint (Skill) that the Agent can invoke. The Agent focuses on intent; COGTOME handles execution rigor.

```
Agent intent  →  COGTOME Skill  →  Executed with guarantees
                 (DAG + contracts)
```

---

## Key Features

| Feature | Description |
|---------|-------------|
| **Process Isolation** | Each tool runs in a separate OS process with timeout and sandbox |
| **Layered Sandbox Isolation** | SandboxBackend trait with 4 backends: bubblewrap, e2b, quickjs, none |
| **Zero-Rewrite Adapter** | Any script with JSON stdin/stdout becomes a Unit |
| **JSON Schema Contracts** | Input/output validation |
| **DAG Workflows** | Motifs support `if` branches, `foreach` loops, parallel execution |
| **MCP Bridge** | Run MCP Servers as COGTOME Units |

---

## Architecture

COGTOME uses a three-layer execution model:

```
Agent (natural language intent)
        │
        ▼
┌─────────────────────┐
│       Skill         │  ← Agent-facing layer
│                     │     Name, description, input/output schema
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│       Motif         │  ← Orchestration logic (JSON DAG)
│                     │     Nodes: start, unit, if, match, foreach, fork, join, return
└─────────┬───────────┘
          │ IPC (fork+exec, stdin/stdout JSON)
          ▼
┌─────────────────────┐
│        Unit         │  ← Atomic execution (independent process)
│                     │     Any language, JSON stdin/stdout
└─────────────────────┘
```

### Layer Overview

| Layer | Purpose | Agent Visible? |
|-------|---------|---------------|
| **Skill** | Exposed capability with description and schema | ✅ Yes |
| **Motif** | JSON DAG orchestration | ❌ No |
| **Unit** | Atomic executable process | ❌ No |

### Core Discipline

1. **Units never call each other** — Runtime blocks recursive invocation via `COGTOME_UNIT_MODE=1`.
2. **All cross-layer calls go through Runtime IPC** — No direct coupling.
3. **Schema validation at every boundary** — Fail fast on bad inputs.

### SandboxBackend Trait

COGTOME defines a `SandboxBackend` trait for pluggable isolation. Each Unit declares its isolation level in `unit.json`:

```json
{
  "name": "my-unit",
  "isolation": "bubblewrap",
  "entry": "bin/my-unit"
}
```

| Backend | Isolation Level | Use Case |
|---------|----------------|----------|
| `bubblewrap` | Local namespace sandbox | Default for most Units |
| `e2b` | Remote strong isolation | Untrusted code, network-sensitive |
| `quickjs` | Ultra-lightweight JS sandbox | Simple JS scripts |
| `none` | No sandbox (fallback) | Trusted local tools |

---

## Sandbox Isolation Strategy

COGTOME separates *what* to execute from *where* to execute. The isolation layer delegates to purpose-built sandbox runtimes rather than reimplementing cgroup/seccomp logic.

### Layering Logic

1. **Unit declares isolation** in `unit.json` (or default from `cogtome.toml`).
2. **Runtime resolves backend** via the `isolation` field.
3. **Sandbox wraps execution** — fork+exec happens inside the chosen backend.
4. **Fallback chain** — if a backend is unavailable, COGTOME falls back to `none` with a warning.

```toml
# cogtome.toml
[units.defaults]
isolation = "bubblewrap"

[units.isolation.my-untrusted-unit]
backend = "e2b"
e2b_api_key = "${E2B_API_KEY}"
```

### Threat Model Coverage

| Threat | bubblewrap | e2b | quickjs | none |
|--------|-----------|-----|---------|------|
| Filesystem escape | ✅ | ✅ | ✅ | ❌ |
| Network access | ✅ | ✅ | ✅ | ❌ |
| Process tree escape | ✅ | ✅ | ✅ | ❌ |
| Kernel exploit | ❌ | ✅ | ❌ | ❌ |

---

## Quick Start

### 1. Build

```bash
git clone https://github.com/haodonLiu/cogtome.git
cd cogtome
cargo build --release
```

### 2. Run Examples

```bash
./target/release/cogtome discover
./target/release/cogtome run text-processing --input '{"text":"hello"}'
./target/release/cogtome motif run browser-fetch --input '{"url":"https://example.com"}'
./target/release/cogtome unit run text-uppercase --input '{"text":"hello"}'
```

### 3. MCP Bridge

```bash
./target/release/cogtome mcp-bridge \
  --server "npx -y @modelcontextprotocol/server-filesystem /tmp" \
  --tool list_allowed_directories
```

### 4. Environment Variables

```bash
export COGTOME_SKILLS_DIR=./skills   # Skills directory (default: ./skills)
export COGTOME_TIMEOUT=60            # Unit execution timeout (default: 30s)
```

---

## Project Structure

```
cogtome/
├── src/                    # Runtime source (Rust)
│   ├── main.rs             # CLI entry point (clap)
│   ├── api.rs              # HTTP API server (axum)
│   ├── assembly.rs         # Assembly registry
│   ├── mcp_server.rs       # MCP Server (JSON-RPC 2.0)
│   ├── discovery.rs        # Skills directory scanning
│   ├── config.rs           # cogtome.toml parsing
│   ├── engine/             # Execution engine
│   │   ├── mod.rs          # GraphMotifEngine + StructureExecutor
│   │   ├── graph.rs        # Graph validation
│   │   ├── unit_runner.rs  # Unit execution (fork+exec)
│   │   └── mcp_bridge.rs   # MCP Bridge
│   └── context/            # Execution context
│       ├── expression.rs   # Expression evaluation
│       └── variables.rs    # Variable resolution
├── skills/                 # Skills directory (runtime-loaded)
│   ├── units/<name>/bin/   # Atomic executables
│   ├── motifs/<name>.json  # JSON Motif DAG
│   └── <complex>/SKILL.md  # Complex definitions
├── assemblies/             # MCP Server assemblies
│   └── <name>/
│       ├── manifest.json
│       └── workflow.json   # MotifManifestV2 DAG
└── cogtome.toml            # Runtime configuration
```

---

## CLI Reference

```bash
cogtome discover                              # Scan all Complexes
cogtome run <complex> --input <json>         # Run Complex
cogtome motif run <name> --input <json>      # Run Motif
cogtome structure run <name> --input <json>  # Run Structure
cogtome unit run <name> --input <json>       # Run Unit
cogtome serve --port 8080                    # Start REST API
cogtome mcp-bridge --server <cmd> --tool <name>  # Run MCP Server as Unit
cogtome mcp-server --assemblies <dir>        # Start MCP Server (stdio mode)
cogtome pack <skill>                         # Package to .cogtome
cogtome install <file.cogtome>              # Install package
cogtome reload                               # Hot reload
cogtome validate <path>                      # Validate manifest
cogtome stats                                # Assembly call heatmap
```

---

## Comparison

| Feature | COGTOME | E2B | MCP | LangChain | Dify/n8n |
|---------|---------|-----|-----|-----------|----------|
| **Primary goal** | Run existing scripts safely | Cloud sandbox for AI code | Protocol standard | Python framework | Human workflow |
| **Tool rewrite required** | ❌ No | ⚠️ Python SDK | ✅ Yes | ⚠️ Python wrapper | ⚠️ Usually yes |
| **Process isolation** | ✅ Layered backends | ✅ MicroVM | Depends on host | ❌ In-process | ✅ Server |
| **Sandbox options** | 4 backends | Single (Firecracker) | None | None | None |
| **Agent-native interface** | ✅ CLI | ✅ Python/JS SDK | Protocol | Python API | GUI/API |
| **Best for** | Local script sandboxing | Remote untrusted code | Cross-platform tools | Python app integration | Business automation |

---

## Design Principles

1. **Don't make users learn metaphors** — Call things what they are: Units, Motifs, Skills.
2. **Zero-rewrite adoption** — Your existing scripts are valuable. Preserve them.
3. **Isolation by default** — Every tool runs in its own process. No exceptions.
4. **Schema contracts** — JSON Schema validation at every boundary.
5. **MCP compatibility** — We don't compete with MCP; we run it.
6. **Open Source First** — Pure Rust, no closed-source dependencies. Every line auditable.
7. **Isolation Outsourcing** — Don't reinvent seccomp. Delegate to bubblewrap, e2b, quickjs via a trait.

---

## Phase Status

### Phase 1: Core Runtime ✅

- [x] Four-layer execution model (Complex → Structure → Motif → Unit)
- [x] CLI framework (discover, run, unit/motif/structure)
- [x] Unit execution (fork+exec, stdin/stdout JSON, timeout, temp sandbox)
- [x] JSON Motif parsing and execution (DAG graph)
- [x] Complex discovery (SKILL.md front-matter parsing)
- [x] `foreach` loop, `if` conditional, error strategies
- [x] HTTP API server, Pack/Install, MCP Bridge, MCP Server
- [x] Assembly registry, call heatmap, graceful shutdown

### Phase 2: Usability 🔧

- [ ] Integration test coverage (test_suite/)
- [ ] `cogtome run` stable 100-run verification
- [ ] Motif inline script nodes, `cogtome wrap` migration tool
- [ ] Layered sandbox backends (bubblewrap, e2b, quickjs)

### Phase 3: Observability 📊

- [ ] Execution trace logging, Checkpoint nodes, Prometheus metrics

### Phase 4: Integration 🔗

- [ ] KimiCLI bridge, OpenClaw gateway, file system auto-reload, Skill registry

---

## Links

- [User Manual](./docs/USER_MANUAL.md)
- [Technical Specification](./development/TECHNICAL_SPEC.md)
- [Skill Authoring Guide](./development/SKILL_AUTHORING_GUIDE.md)

---

## License

MIT

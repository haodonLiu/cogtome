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
4. [Quick Start](#quick-start)
5. [Project Structure](#project-structure)
6. [CLI Reference](#cli-reference)
7. [Web UI](#web-ui)
8. [Comparison](#comparison)
9. [Design Principles](#design-principles)

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
| **Zero-Rewrite Adapter** | Any script with JSON stdin/stdout becomes a Unit |
| **JSON Schema Contracts** | Input/output validation |
| **DAG Workflows** | Motifs support `if` branches, `foreach` loops, parallel execution |
| **MCP Bridge** | Run MCP Servers as COGTOME Units |
| **Visual Editor** | Web UI with drag-and-drop graph editor |

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
# Discover all Complexes
./target/release/cogtome discover

# Run a Complex
./target/release/cogtome run text-processing --input '{"text":"hello"}'

# Run a Motif directly
./target/release/cogtome motif run browser-fetch --input '{"url":"https://example.com"}'

# Run a Unit directly
./target/release/cogtome unit run text-uppercase --input '{"text":"hello"}'
```

### 3. MCP Bridge

```bash
# Run an MCP Server as a COGTOME Unit
./target/release/cogtome mcp-bridge \
  --server "npx -y @modelcontextprotocol/server-filesystem /tmp" \
  --tool list_allowed_directories
```

### 4. Environment Variables

```bash
# Skills directory (default: ./skills)
export COGTOME_SKILLS_DIR=./skills

# Unit execution timeout (default: 30s)
export COGTOME_TIMEOUT=60
```

---

## Project Structure

```
cogtome/
├── src/                    # Runtime source (Rust)
│   ├── main.rs             # CLI entry point (clap)
│   ├── api.rs              # HTTP API server (axum)
│   ├── discovery.rs        # Skills directory scanning
│   ├── config.rs           # cogtome.toml parsing
│   ├── engine/             # Execution engine
│   │   ├── mod.rs          # GraphMotifEngine + StructureExecutor
│   │   ├── graph.rs        # Graph validation
│   │   ├── unit_runner.rs  # Unit execution (fork+exec)
│   │   └── mcp_bridge.rs  # MCP Bridge
│   └── context/            # Execution context
│       ├── expression.rs   # Expression evaluation
│       └── variables.rs    # Variable resolution
├── webui/                  # Web UI (React + React Flow)
├── skills/                 # Skills directory (runtime-loaded)
│   ├── units/<name>/bin/   # Atomic executables
│   ├── motifs/<name>.json  # JSON Motif DAG
│   └── <complex>/SKILL.md  # Complex definitions
└── cogtome.toml            # Runtime configuration
```

---

## CLI Reference

```bash
# Discovery
cogtome discover                              # Scan all Complexes

# Execution
cogtome run <complex> --input <json>         # Run Complex
cogtome motif run <name> --input <json>       # Run Motif
cogtome structure run <name> --input <json>  # Run Structure
cogtome unit run <name> --input <json>       # Run Unit

# HTTP API
cogtome serve --port 8080                    # Start REST API

# MCP
cogtome mcp-bridge --server <cmd> --tool <name>  # Run MCP Server as Unit
cogtome mcp-server --assemblies <dir>        # Start MCP Server (stdio mode)

# Pack & Install
cogtome pack <skill>                         # Package to .cogtome
cogtome install <file.cogtome>              # Install package

# Utility
cogtome reload                               # Hot reload
cogtome validate <path>                      # Validate manifest
cogtome stats                                # Assembly call heatmap
```

---

## Web UI

COGTOME includes a **visual studio** for creating and debugging Motifs.

### Running the Web UI

```bash
# One-click start
./start-webui.sh

# Or manual
cargo build --release
./target/release/cogtome serve --port 3334 &
cd webui && npm install && npm run dev
```

Access at **http://localhost:3333**

### Features

- **Graph editor**: Drag-and-drop composition with 9 node types
- **Auto-layout**: Grid-based automatic node positioning
- **Execution trace**: See data flow through each step

---

## Comparison

| Feature | COGTOME | MCP | LangChain | Dify/n8n |
|---------|---------|-----|-----------|----------|
| **Primary goal** | Run existing scripts safely | Protocol standard | Python framework | Human workflow |
| **Tool rewrite required** | ❌ No | ✅ Yes | ⚠️ Python wrapper | ⚠️ Usually yes |
| **Process isolation** | ✅ Yes | Depends on host | ❌ In-process | ✅ Server |
| **Agent-native interface** | ✅ CLI | Protocol | Python API | GUI/API |
| **Best for** | Local script sandboxing | Cross-platform tools | Python app integration | Business automation |

---

## Design Principles

1. **Don't make users learn metaphors** — Call things what they are: Units, Motifs, Skills.
2. **Zero-rewrite adoption** — Your existing scripts are valuable. Preserve them.
3. **Isolation by default** — Every tool runs in its own process. No exceptions.
4. **Schema contracts** — JSON Schema validation at every boundary.
5. **MCP compatibility** — We don't compete with MCP; we run it.
6. **Visual + Textual** — Both graph editor and JSON authoring supported.

---

## Links

- [User Manual](./docs/USER_MANUAL.md)
- [Technical Specification](./development/TECHNICAL_SPEC.md)
- [Skill Authoring Guide](./development/SKILL_AUTHORING_GUIDE.md)

---

## License

MIT

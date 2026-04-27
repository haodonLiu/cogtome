<img src="cover.jpg" width="400" alt="COGTOME" />

> English | [中文版本](README_CN.md)

# COGTOME

> **Process-level sandbox executor for Agent tools.**
>
> COGTOME runs your existing scripts and binaries as isolated, contract-governed tools for AI Agents. No rewrites, no framework lock-in.

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
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
8. [Roadmap](#roadmap)
9. [Design Principles](#design-principles)

---

## What is COGTOME

COGTOME is a **runtime that turns any executable into an Agent-callable tool** with process isolation, JSON Schema contracts, and lifecycle management.

### The Problem

Agents need to call external tools, but today the choices are poor:

- **Direct subprocess**: No isolation. A runaway script can hang or crash your Agent.
- **MCP**: Great protocol, but you must rewrite tools as MCP Servers. Your existing Python/Bash scripts don't work out of the box.
- **Workflow engines** (Dify, n8n): Built for humans clicking buttons, not for Agents calling APIs.

COGTOME sits between your existing tools and your Agent:

```
Your existing scripts  →  COGTOME Runtime  →  Agent
(Python, Bash, anything)   (isolation + schema)   (semantic CLI)
```

### What makes it different

| Capability | How COGTOME handles it |
|-----------|------------------------|
| **Existing scripts** | Any executable that reads JSON stdin / writes JSON stdout works. Zero rewrites. |
| **Isolation** | Every tool runs in a separate `fork+exec` process with a temp sandbox. |
| **Contracts** | JSON Schema input/output validation at the boundary. |
| **MCP ecosystem** | Planned bridge to run MCP Servers as first-class Units ([see Roadmap](#roadmap)). |
| **Orchestration** | Declarative YAML workflows (Motifs) chain tools with `if`, `foreach`, `retry`. |

---

## Key Features

**🔒 Process Isolation** — Each tool execution is a separate OS process with timeout, temp-dir sandbox, and optional env whitelist. A buggy Unit cannot crash the Runtime or another Unit.

**🛠 Zero-Rewrite Tool Adapter** — Your Python script, Bash one-liner, or compiled binary becomes a Unit by reading JSON from stdin and writing JSON to stdout. No SDK, no protocol adapters.

**📐 JSON Schema Contracts** — Define inputs/outputs with JSON Schema. The Runtime validates before execution and type-checks the result.

**🧩 Declarative Workflows** — Chain Units into Motifs with YAML: sequential steps, `if` branches, `foreach` loops, parallel execution, and aggregate results.

**🎨 Low-Code Skill Creation** — Web UI with drag-and-drop graph editor for visually composing Motifs and assembling Skills. Non-developers can build reusable Skills without writing YAML.

**🎯 Semantic CLI** — Agents interact via human-meaningful commands (`read file`, `fetch webpage`) rather than raw shell (`cat /path`, `curl url`).

**🌉 MCP Bridge (Planned)** — Run existing MCP Servers inside COGTOME without rewriting them, solving the ecosystem cold-start problem.

---

## Architecture

COGTOME uses a three-layer execution model:

```
Agent (natural language intent)
        │
        ▼
┌─────────────────────┐
│       Skill         │  ← Agent-facing unit. Has name, description, input/output schema.
│   (Business Unit)   │     Internally a Motif or a direct Unit reference.
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│       Motif         │  ← Orchestration logic. YAML declarative flow.
│    (Workflow)       │     Steps reference Units. Supports foreach, if, retry, on_error.
└─────────┬───────────┘
          │ IPC (fork+exec, stdin/stdout JSON)
          ▼
┌─────────────────────┐
│        Unit         │  ← Atomic execution. Independent process.
│    (Executable)     │     Any language. Reads JSON from stdin, prints JSON to stdout.
└─────────────────────┘
```

### Layer Overview

| Layer | Purpose | Agent Visible? |
|-------|---------|---------------|
| **Skill** | Exposed capability with description and schema | ✅ Yes |
| **Motif** | Orchestrates Units into reusable workflows | ❌ No |
| **Unit** | Atomic executable | ❌ No |

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
# Discover all Skills
./target/release/cogtome discover

# Run a Skill
./target/release/cogtome run text-processing --input '{"text":"hello"}'

# Run Motif directly
./target/release/cogtome motif run text-transform --input '{"text":"hello"}'

# Run Unit directly
./target/release/cogtome unit run text-uppercase --input '{"text":"hello"}'
```

### 3. Wrap your own script (planned)

```bash
# One-command wrap (coming soon)
cogtome wrap ./my_script.py --name my-analyzer
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
│   ├── discovery.rs        # Directory scanning
│   ├── config.rs           # Config file loading
│   ├── context/            # Execution context
│   │   ├── mod.rs
│   │   ├── expression.rs   # Expression evaluation
│   │   └── variables.rs    # Variable resolution
│   └── engine/             # Execution engine
│       ├── mod.rs          # MotifEngine + StructureExecutor
│       ├── motif_manifest.rs # Manifest types
│       ├── unit_runner.rs  # Unit execution (fork+exec)
│       └── foreach.rs      # Foreach executor
├── webui/                  # Web UI (React + React Flow + TypeScript)
│   ├── src/
│   │   ├── components/     # React components
│   │   ├── store/          # Zustand state
│   │   └── api/            # API client
│   └── dist/               # Built static assets
├── skills/                 # Skills directory (runtime-loaded)
│   ├── units/<name>/bin/   # Atomic executables
│   ├── motifs/<name>.yaml  # YAML workflow motifs
│   ├── structures/<name>/  # Business structures (to be merged into Skill)
│   └── <complex>/          # Complex definitions (to be merged into Skill)
│       └── SKILL.md
├── Cargo.toml
└── cogtome.toml            # Runtime configuration
```

---

## CLI Reference

### Execution Commands

```bash
# Discovery
cogtome discover                              # Scan all Skills

# Run (Skill → Motif → Unit)
cogtome run <skill> --input <json>            # Run Skill
cogtome motif run <name> --input <json>       # Run Motif
cogtome unit run <name> --input <json>        # Run Unit

# HTTP API Server
cogtome serve --port 8080                     # Start REST API

# Pack & Install
cogtome pack <skill>                          # Package to .cogtome
cogtome install <file.cogtome>                # Install package

# Utility
cogtome validate                              # Validate all skills
cogtome reload                                # Hot reload skills
cogtome help                                  # Show all commands
```

---

## Web UI

COGTOME includes a **visual Skill studio** for both creating and debugging Motifs.

### Skill Creation

- **Graph editor**: Drag-and-drop composition of Motifs with 9 node types (start, unit, if, match, foreach, fork, join, return, motif)
- **Graph ↔ YAML sync**: Visual editing with automatic YAML serialization
- **Auto-layout**: Grid-based automatic node positioning

### Execution Debugger

- **Execution trace**: See data flow through each step (which node is stuck, what are the inputs/outputs)
- **Unit test panel**: Quick-run a single Unit with custom parameters
- **Live graph view**: Visualize the Motif DAG during or after execution

### Running the Web UI

```bash
# One-click start (builds Rust + API + WebUI)
./start-webui.sh

# Or manual
cargo build --release
cogtome serve --port 3334 &
cd webui && npm install && npm run dev
```

Access at **http://localhost:3333**

---

## Comparison

| Feature | COGTOME | MCP | LangChain | Dify/n8n |
|---------|---------|-----|-----------|----------|
| **Primary goal** | Run existing scripts safely | Protocol standard | Python framework | Human workflow |
| **Tool rewrite required** | ❌ No | ✅ Yes (MCP Server) | ⚠️ Python wrapper | ⚠️ Usually yes |
| **Process isolation** | ✅ Yes | Depends on host | ❌ In-process | ✅ Server |
| **Agent-native interface** | ✅ CLI | Protocol | Python API | GUI/API |
| **Best for** | Local script sandboxing | Cross-platform tools | Python app integration | Business automation |

---

## Roadmap

### Phase 1: Stabilize (Current)

- [x] CLI framework with discover, run, unit/motif/skill run
- [x] Unit execution (fork+exec, stdin/stdout JSON, timeout, temp sandbox)
- [x] YAML Motif parsing and execution
- [x] Skill discovery (SKILL.md front-matter parsing)
- [x] `foreach` loops with aggregate
- [x] `if` conditional execution
- [x] Retry with backoff
- [x] Error strategies (fail, continue, fallback)
- [x] HTTP API server
- [x] Pack/install with tar.gz

### Phase 2: MCP & Ergonomics (0–6 weeks)

- [ ] **MCP Bridge Unit** — run MCP Servers as COGTOME Units via stdio JSON-RPC
- [ ] **Skill layer merge** — collapse Structure + Complex into a single Skill concept
- [ ] **Inline script nodes** — run Python/Bash snippets inside Motifs without standalone Units
- [ ] **`cogtome wrap`** — one-command migration from existing scripts
- [ ] **Docker Unit Runner** — optional containerized execution for untrusted tools

### Phase 3: Observability & Integration (6–12 weeks)

- [ ] Execution trace logging (full input/output/history per run)
- [ ] Checkpoint/resume for long-running Motifs
- [ ] Prometheus metrics export
- [ ] KimiCLI bridge (Wire/ACP long-connection mode)
- [ ] OpenClaw gateway bridge (WebSocket)

### Phase 4: Ecosystem

- [ ] File-system auto-reload (notify crate)
- [ ] Skill registry / marketplace
- [ ] Web UI execution debugger (trace view)

---

## Design Principles

1. **Don't make users learn metaphors** — Call things what they are: Units, Workflows, Skills.
2. **Zero-rewrite adoption** — Your existing scripts are valuable. Preserve them.
3. **Isolation by default** — Every tool runs in its own process. No exceptions.
4. **Schema contracts** — JSON Schema validation at every boundary.
5. **MCP compatibility** — We don't compete with MCP; we run it.
6. **Visual + Textual** — Both graph editor and YAML authoring supported. Debuggability and creation ergonomics are equally important.

---

## Links

- [Technical Specification](./development/TECHNICAL_SPEC.md)
- [Implementation Guide](./development/IMPLEMENTATION_GUIDE.md)
- [Skill Authoring Guide](./development/SKILL_AUTHORING_GUIDE.md)

---

## License

MIT

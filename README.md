<img src="cover.jpg" width="400" alt="COGTOME" />

> English | [中文版本](README_CN.md)

# COGTOME

> **Gears turn the tome, mechanics execute the craft.**
>
> COGTOME is a micro operating system and execution runtime for AI Agents.
> Agents forge gears (Unit), assemble gear trains (Motif), package drive trains (Structure), and compile domain tomes (Complex).
> The Runtime handles discovery, compilation, scheduling, execution, and reclamation.

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## Table of Contents

1. [What is COGTOME](#what-is-cogtome)
2. [Key Highlights](#key-highlights)
3. [Core Architecture: Four-Layer Model](#core-architecture-four-layer-model)
4. [Quick Start](#quick-start)
5. [Project Structure](#project-structure)
6. [CLI Reference](#cli-reference)
7. [Web UI](#web-ui)
8. [Roadmap](#roadmap)
9. [Design Principles](#design-principles)

---

## What is COGTOME

COGTOME is **not** a framework, **not** a library — it is an **independent process-level runtime**: a micro operating system for Agents.

| OS Concept | COGTOME Equivalent |
|-----------|-------------------|
| Kernel | COGTOME Runtime (Rust) |
| User Process | Agent (LLM / Program) |
| System Call | Unit (atomic execution) |
| User-space Function | Motif (orchestration logic) |
| Application | Structure (business encapsulation) |
| App Store | Complex (domain facade) |
| Shell | `cogtome` CLI |
| GUI | Web UI (React Flow) |

### The Core Problem

Agents need to call external tools, but direct `subprocess` calls cause:
- Process management chaos (leaks, zombie processes)
- No type safety (no input/output contracts)
- No versioning or discovery mechanism
- No execution trace tracking

COGTOME solves this: Agents write **business logic**, Runtime handles **infrastructure**.

### Brand Metaphors

| Technical Term | Metaphor | Meaning |
|---------------|----------|---------|
| Unit | Cog | The indivisible atomic executor |
| Motif | Gear Assembly | Orchestration of cogs |
| Structure | Drive Train | Completes a business goal |
| Complex | Tome | Domain book holding drive trains |

---

## Key Highlights

**🎯 Agent-Native CLI System** — COGTOME is designed **for Agents, by Agents**. Agents interact via pure CLI with semantic commands ("read file", "fetch webpage") rather than raw shell commands ("cat /path", "curl url"). No human-in-the-loop required.

**🧩 Layered Abstraction** — Four-layer model (Unit → Motif → Structure → Complex) provides clear separation between atomic execution and business logic. Agents focus on "what", Runtime handles "how".

**🎨 Low-Code Skill Creation** — Web UI with drag-and-drop React Flow editor enables visual composition of Motifs and Structures. Humans can build Skills without writing code, agents consume them via CLI.

**🔌 Protocol-Agnostic** — Unlike MCP servers that require protocol adaptation per tool, COGTOME Units are language-agnostic executables. Any program that speaks JSON stdin/stdout works out of the box.

**🏗️ Zero Business Logic in Runtime** — The COGTOME binary itself contains no built-in tools. All capabilities come from Skills—true separation of concerns.

---

## Comparison

| Feature | COGTOME | MCP Servers | LangChain | Dify/n8n |
|---------|---------|-------------|-----------|-----------|
| **Primary User** | Agent | Agent | Developer | Human |
| **Interface** | Pure CLI | Protocol | Python API | GUI |
| **Skill Creation** | CLI + Web UI | Code Required | Code Required | Visual |
| **For Agents** | ✅ Native | ⚠️ Adapter | ❌ Library | ❌ Human |
| **Runtime Model** | Process Isolated | Protocol | In-process | Server |
| **Contracts** | JSON Schema | JSON-RPC | Python Types | Form-based |

---

## Core Architecture: Four-Layer Model

```
Agent (natural language intent)
        │
        ▼
┌─────────────────────┐
│      Complex        │  ← Only layer visible to Agent
│   (Domain Tome)     │     Has description, auto-discovered
│                     │
│  select_structure() │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│     Structure        │  ← Business black box
│   (Drive Train)      │     manifest.json defines contracts
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│       Motif          │  ← Orchestration logic (JSON DAG)
│   (Gear Assembly)    │     start/unit/if/match/foreach/fork/join/return
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│        Unit          │  ← Atomic execution
│       (Cog)          │     stdin/stdout JSON, fork+exec
└─────────────────────┘
```

### Layer Overview

| Layer | Name | Agent Visible? | Essence |
|-------|------|---------------|---------|
| **L4** | **Complex** | ✅ Only visible | Domain facade with description |
| **L3** | **Structure** | ❌ Hidden | Business structure |
| **L2** | **Motif** | ❌ Hidden | Orchestrates Units (JSON DAG) |
| **L1** | **Unit** | ❌ Hidden | Atomic executor |

### Supported Node Types (v2.0)

| Node | Purpose |
|------|---------|
| `start` | Entry point (required, exactly one) |
| `unit` | Execute atomic Unit |
| `if` | Conditional branching (true/false) |
| `match` | Multi-way branching |
| `foreach` | Loop with optional subgraph |
| `fork` | Parallel branch split |
| `join` | Parallel branch sync |
| `return` | Output values (required, at least one) |

### Core Discipline

1. **Units never call each other** (Runtime blocks via `COGTOME_UNIT_MODE=1`)
2. **Motifs don't directly call each other** (composed via Structure)
3. **Structure doesn't directly call Unit** (must go through Motif)
4. **Complex is the only layer with `description`**
5. **All cross-layer calls go through Runtime IPC**

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

# Run Complex (complete domain Skill)
./target/release/cogtome run text-processing --input '{"text":"hello"}'

# Run Motif (JSON DAG orchestration)
./target/release/cogtome motif run text-transform --input '{"text":"hello"}'

# Run Structure
./target/release/cogtome structure run text-pipeline --input '{"text":"hello"}'

# Run Unit directly
./target/release/cogtome unit run text-uppercase --input '{"text":"hello"}'
```

### 3. Environment Variables

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
│   ├── discovery.rs         # Directory scanning
│   ├── config.rs           # Config file loading
│   ├── context/             # Execution context
│   │   ├── mod.rs
│   │   ├── expression.rs    # Expression evaluation
│   │   └── variables.rs     # Variable resolution
│   └── engine/              # Execution engine
│       ├── mod.rs           # GraphMotifEngine (JSON DAG)
│       ├── graph.rs          # Graph/Node/Edge + validation
│       ├── motif_manifest.rs # Motif/Structure types
│       ├── unit_runner.rs    # Unit execution (fork+exec)
│       └── foreach.rs        # Foreach executor
├── webui/                   # Web UI (React + React Flow + TypeScript)
│   ├── src/
│   │   ├── components/      # React components
│   │   │   ├── editors/     # MotifEditor, StructureEditor, UnitEditor
│   │   │   └── graph/       # 9 node types (start/unit/if/match/...)
│   │   ├── store/           # Zustand state
│   │   └── api/             # API client
│   └── dist/                # Built static assets
├── skills/                  # Skills directory (runtime-loaded)
│   ├── units/<name>/bin/    # Atomic executables
│   ├── motifs/<name>.json    # JSON DAG motifs
│   ├── structures/<name>/manifest.json
│   └── <complex>/SKILL.md
├── Cargo.toml
└── cogtome.toml            # Runtime configuration
```

---

## CLI Reference

### Execution Commands

```bash
# Discovery
cogtome discover                              # Scan all Complexes

# Run (Complex → Structure → Motif → Unit)
cogtome run <complex> --input <json>          # Run Complex
cogtome structure run <name> --input <json>   # Run Structure
cogtome motif run <name> --input <json>       # Run Motif (JSON DAG)
cogtome unit run <name> --input <json>       # Run Unit

# HTTP API Server
./start-webui.sh                              # One-click: API + WebUI
cogtome serve --port 3334                     # API only on port 3334

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

COGTOME includes a **visual graph editor** for Motifs and Structures using React Flow.

### Screenshots

| Editor | Description |
|--------|-------------|
| **Motif Editor** | Graph canvas with 9 node types |
| **Structure Editor** | Visual graph editor for assembling Motifs |
| **Unit Editor** | Test panel for Unit execution |

### Running the Web UI

```bash
# One-click start (builds Rust + API on 3334 + WebUI on 3333)
./start-webui.sh

# Or manual
cargo build --release
cogtome serve --port 3334 &
cd webui && npm install && npm run dev
```

Access at **http://localhost:3333**

### Features

- **Graph ↔ JSON Sync**: Visual editing with automatic JSON serialization
- **9 Node Types**: start/unit/if/match/foreach/fork/join/return/motif
- **Auto-layout**: Grid-based automatic node positioning
- **Keyboard shortcuts**: Ctrl+S save, Delete remove
- **Dark theme**: Default dark UI

---

## JSON Motif Format (v2.0)

Motifs are stored as JSON DAGs:

```json
{
  "name": "text-transform",
  "type": "motif",
  "version": "2.0",
  "graph": {
    "nodes": [
      { "id": "start", "type": "start", "position": { "x": 0, "y": 0 }, "data": {} },
      { "id": "upper", "type": "unit", "position": { "x": 200, "y": 0 }, "data": { "unit": "text-uppercase" } },
      { "id": "return", "type": "return", "position": { "x": 400, "y": 0 }, "data": { "values": { "result": "${steps.upper.output" } } }
    ],
    "edges": [
      { "source": "start", "target": "upper" },
      { "source": "upper", "target": "return" }
    ]
  }
}
```

### Validation Rules

- Exactly one `start` node
- At least one `return` node
- No cycles (DAG required)
- All nodes reachable from start
- Conditional nodes require labeled edges

---

## Built-in Skills

| Complex | Structures | Description |
|---------|-----------|-------------|
| `core-tools` | `shell-executor`, `file-read`, `file-write` | OpenClaw tool wrappers |
| `web-fetch` | `fetch` | HTTP content fetching |
| `text-processing` | `text-pipeline` | Text transformation |

---

## Design Principles

1. **Runtime has zero business logic** — COGTOME binary has no built-in Units
2. **Agent authoring freedom** — Units any language, Motifs in JSON DAG
3. **Strong contracts** — JSON Schema validation at each layer
4. **Process isolation** — Units never call each other
5. **Observability** — Complete execution chain logging
6. **Visual + Textual** — Both graph editor and JSON authoring supported

---

## Links

- [Technical Specification](./development/TECHNICAL_SPEC.md) — Detailed architecture
- [OS Metaphors](./development/OS_METAPHORS.md) — Conceptual foundation

---

## License

MIT

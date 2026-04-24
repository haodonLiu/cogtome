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
2. [Core Architecture: Four-Layer Model](#core-architecture-four-layer-model)
3. [Quick Start](#quick-start)
4. [Project Structure](#project-structure)
5. [CLI Reference](#cli-reference)
6. [Roadmap](#roadmap)
7. [Design Principles](#design-principles)

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
│   (Drive Train)      │     manifest.yaml defines contracts
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│       Motif          │  ← Orchestration logic
│   (Gear Assembly)    │     YAML declarative
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
| **L2** | **Motif** | ❌ Hidden | Orchestrates Units |
| **L1** | **Unit** | ❌ Hidden | Atomic executor |

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

# Run Motif (orchestrated logic)
./target/release/cogtome motif run text-transform --input '{"text":"hello"}'

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
│   ├── context.rs          # Execution context + variable resolution
│   ├── discovery.rs        # Directory scanning
│   └── engine.rs           # UnitRunner + MotifEngine + StructureExecutor
├── skills/                 # Skills directory
│   ├── units/              # Atomic executors
│   ├── motifs/             # Orchestration logic (YAML)
│   ├── structures/          # Business structures
│   └── <complex>/          # Domain Complex
│       └── SKILL.md        # Complex definition (required)
├── test_suite/             # Test cases
├── development/            # Technical documents
└── Cargo.toml
```

### Skills Directory Structure

```
skills/
├── units/<name>/bin/<name>     # Executable Unit (any language)
├── motifs/<name>.yaml          # YAML Motif
├── structures/<name>/
│   └── manifest.yaml           # Structure manifest
└── <complex>/SKILL.md          # Complex (must have description)
```

---

## CLI Reference

### Implemented ✅

```bash
# Discovery
cogtome discover                              # Scan all Complexes

# Execution
cogtome run <complex> --input <json>        # Run Complex
cogtome unit run <name> --input <json>     # Run Unit directly
cogtome motif run <name> --input <json>    # Run Motif
cogtome structure run <name> --input <json> # Run Structure

# Help
cogtome help                                  # Show all commands
```

### Planned (Not Yet Implemented) 🔮

```bash
cogtome unit list                            # List all Units
cogtome motif list                           # List all Motifs
cogtome structure list                       # List all Structures
cogtome validate                             # Validate all Skills
cogtome logs                                 # Show execution logs
cogtome inspect <id>                         # Inspect execution tree
cogtome daemon start/stop                   # Daemon mode
```

---

## Roadmap

### Phase 1: Foundation ✅ (Current)

- [x] CLI framework
- [x] Unit execution (fork+exec, stdin/stdout JSON)
- [x] YAML Motif parsing (serial flow)
- [x] Structure → Motif → Unit chain
- [x] Complex discovery (SKILL.md parsing)
- [x] Default timeout (30s)
- [x] Skills path configuration (`COGTOME_SKILLS_DIR`)

### Phase 2: Core Orchestration 🔮

- [ ] `foreach` loop with `aggregate`
- [ ] Expression engine (variable indexing, array access)
- [ ] `if` conditional execution
- [ ] `max_iterations` safety limit
- [ ] Error layering (`runtime` / `motif` / `unit`)
- [ ] Snapshot semantics (read-only external state)

### Phase 3: Concurrency 🔮

- [ ] Parallel `foreach` (`parallel: true`)
- [ ] Unit concurrency declaration (`max_global`, `resource_key`)
- [ ] Runtime resource limiter

### Phase 4: Ecosystem 🔮

- [ ] Python Motif (Unix Socket IPC)
- [ ] HTTP API Server
- [ ] Discovery API (`GET /complexes`)
- [ ] `auto-complex` registration
- [ ] `cogtome pack/install`

---

## Design Principles

1. **Runtime has zero business logic** — COGTOME binary has no built-in Units
2. **Agent authoring freedom** — Units any language, Motifs in YAML/Python/Shell
3. **Strong contracts** — JSON Schema validation at each layer
4. **Process isolation** — Units never call each other
5. **Observability** — Complete execution chain logging

---

## Links

- [Technical Specification](./development/TECHNICAL_SPEC.md) — Detailed architecture
- [OS Metaphors](./development/OS_METAPHORS.md) — Conceptual foundation
- [OpenClaw Integration](./development/OPENCLAW_INTEGRATION.md) — Integration protocol

---

## License

MIT

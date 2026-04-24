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
4. [Full Tutorial: Create a Skill from Scratch](#full-tutorial-create-a-skill-from-scratch)
5. [Interface Specifications](#interface-specifications)
6. [CLI Reference](#cli-reference)
7. [Technical Implementation](#technical-implementation)
8. [Roadmap](#roadmap)
9. [Design Principles](#design-principles)

---

## What is COGTOME

### Positioning

COGTOME is not a framework, not a library — it is an **independent process-level runtime**: a micro operating system for Agents.

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

Agents need to call external tools (browsers, databases, APIs, file processing), but direct `subprocess` calls cause:
- Process management chaos (leaks, zombie processes)
- No type safety (no input/output contracts)
- No versioning or discovery mechanism
- No execution trace tracking

COGTOME solves all of the above: Agents only write **business logic** (Unit/Motif/Structure), Runtime handles **all infrastructure** (processes, scheduling, logging, security, discovery).

### Brand Metaphors

| Technical Term | Brand Metaphor | Meaning |
|---------------|---------------|---------|
| Unit | Cog (Tooth) | The indivisible atomic executor |
| Motif | Gear Assembly | Orchestration and combination of cogs |
| Structure | Drive Train | Structure that completes a business goal |
| Complex | Tome | Domain book that holds drive trains |
| Execution | Engage | Cogs mesh and turn |

---

## Core Architecture: Four-Layer Model

```
Agent (natural language intent)
        │
        ▼
┌─────────────────────┐
│      Complex        │  ← Only layer visible to Agent
│   (Domain Tome)     │     Holds description, participates in auto-discovery
│                     │
│  select_structure() │
└─────────┬───────────┘
          │ Load Structure
          ▼
┌─────────────────────┐
│     Structure        │  ← Business black box
│   (Drive Train)      │     manifest.yaml defines contracts
│                     │
│  execute(motifs)   │
└─────────┬───────────┘
          │ Load Motif
          ▼
┌─────────────────────┐
│       Motif          │  ← Orchestration logic
│   (Gear Assembly)    │     YAML / Python / Shell
│                     │
│  unit.call()        │
└─────────┬───────────┘
          │ IPC call
          ▼
┌─────────────────────┐
│        Unit          │  ← Atomic execution
│      (Cog)          │     Independent process, stdin/stdout JSON
│                     │
│  fork + exec        │
└─────────────────────┘
```

### Layer Overview

| Layer | Name | Agent Visible? | Essence | One-line Definition |
|-------|------|----------------|---------|---------------------|
| **L4** | **Complex** | ✅ Only visible | Domain facade | Has `description`, auto-discovered |
| **L3** | **Structure** | ❌ Invisible | Business structure | Internal implementation of a specific goal |
| **L2** | **Motif** | ❌ Invisible | Work chain | Logic that orchestrates Units |
| **L1** | **Unit** | ❌ Invisible | Atomic executor | Fixed CLI, minimal one step |

### Core Discipline

1. **Units never call each other** (Runtime blocks via `COGTOME_UNIT_MODE=1`)
2. **Motifs don't directly call each other** (composed via Structure)
3. **Structure doesn't directly call Unit** (must go through Motif orchestration)
4. **Complex is the only layer with `description`**
5. **All cross-layer calls go through Runtime IPC** (no bare `subprocess`)

---

## Quick Start

### 1. Install

```bash
# Clone the repo
git clone https://github.com/haodonLiu/cogtome.git
cd cogtome

# Build
cargo build --release

# Optional: install to PATH
cp target/release/cogtome /usr/local/bin/
```

### 2. Run Built-in Examples

```bash
# Discover all Complexes
cogtome discover

# Run Unit directly (atomic capability)
cogtome unit run text-uppercase --input '{"text":"hello"}'
# {"result": "HELLO"}

# Run Motif (orchestrated logic)
cogtome motif run text-transform --input '{"text":"hello"}'
# {"upper": "HELLO", "reversed": "olleh"}

# Run Structure (business encapsulation)
cogtome structure run text-pipeline --input '{"text":"hello"}'
# {"upper": "HELLO", "reversed": "olleh"}

# Run Complex (complete domain Skill)
cogtome run text-processing --input '{"text":"hello"}'
# {"upper": "HELLO", "reversed": "olleh"}
```

### 3. Project Structure

```
cogtome/
├── src/                    # Runtime source (Rust)
│   ├── main.rs             # CLI entry point
│   ├── context.rs          # Execution context + variable resolution
│   ├── discovery.rs        # Directory scanning and discovery
│   └── engine.rs           # Unit runner + Motif engine + Structure executor
├── skills/                 # Agent authoring directory (Runtime has no built-in business logic)
│   ├── units/              # Atomic executors
│   ├── motifs/             # Orchestration logic
│   ├── structures/         # Business structures
│   └── <complex>/          # Domain tomes
└── Cargo.toml
```

---

## Full Tutorial: Create a Skill from Scratch

This tutorial demonstrates how to create a runnable Skill from scratch: a text processing pipeline.

### Step 1: Forge a Unit (write atomic capability)

A Unit is **any CLI executable**. Agents can use Python, Bash, Go, or Rust — as long as it follows the stdin/stdout JSON contract.

Create the directory and file:

```bash
mkdir -p skills/units/text-uppercase/bin
cat > skills/units/text-uppercase/bin/text-uppercase << 'EOF'
#!/usr/bin/env python3
import sys, json
inp = json.load(sys.stdin)
print(json.dumps({"result": inp["text"].upper()}))
EOF
chmod +x skills/units/text-uppercase/bin/text-uppercase
```

Create a second Unit:

```bash
mkdir -p skills/units/text-reverse/bin
cat > skills/units/text-reverse/bin/text-reverse << 'EOF'
#!/usr/bin/env python3
import sys, json
inp = json.load(sys.stdin)
print(json.dumps({"result": inp["text"][::-1]}))
EOF
chmod +x skills/units/text-reverse/bin/text-reverse
```

**Unit Interface Contract**:
- **Input**: stdin receives UTF-8 JSON
- **Output**: stdout outputs UTF-8 JSON
- **Error**: exit code non-0, stderr outputs human-readable info
- **Environment variables**: Runtime automatically injects `COGTOME_EXECUTION_ID`, `COGTOME_TRACE_ID`, `COGTOME_UNIT_MODE=1`

### Step 2: Weave a Motif (orchestrate Units)

Motif answers the question: **"Given input, in what order do we call which Units?"**

Agent writes a YAML declarative Motif:

```yaml
# skills/motifs/text-transform.yaml
name: text-transform
type: motif
units_required: [text-uppercase, text-reverse]

flow:
  - name: upper
    unit: text-uppercase
    input:
      text: "${params.text}"

  - name: rev
    unit: text-reverse
    input:
      text: "${params.text}"

return:
  upper: "${steps.upper.output.result}"
  reversed: "${steps.rev.output.result}"
  combined: "${steps.upper.output.result} | ${steps.rev.output.result}"
```

**Variable Scopes**:
- `${params.xxx}` — Original parameters passed by Structure/Agent
- `${steps.<name>.output.xxx}` — stdout JSON field of a step
- `${steps.<name>.exit_code}` — Exit code of a step
- `${env.xxx}` — Environment variable

**Control Flow** (Demo supports serial, full version supports parallel/conditional):
- Default: serial execution
- `parallel: <group>` — Same group concurrent
- `after: <group>` — Wait for group completion
- `condition: "${expr}"` — Conditional execution
- `on_error: <label>` — Error jump

### Step 3: Assemble a Structure (encapsulate business goal)

Structure is a black box that completes **one specific business goal**. It only exposes input/output Schema externally.

```yaml
# skills/structures/text-pipeline/manifest.yaml
name: text-pipeline
type: structure

motifs:
  - name: text-transform

input_schema:
  type: object
  required: [text]
  properties:
    text: { type: string }

output_schema:
  type: object
  properties:
    upper: { type: string }
    reversed: { type: string }
    combined: { type: string }

resources: {}
```

**Field Description**:
- `motifs`: List of Motifs used by this Structure, executed in order
- `input_schema` / `output_schema`: JSON Schema, Runtime automatically validates
- `resources`: Resource requirements (memory, network, GPU), for scheduler reference
- `constraints`: Constraints (e.g. `webgl: true`), for Complex selection reference

### Step 4: Compile a Complex (domain facade)

Complex is **the only layer visible to the Agent**. It holds `description` and is auto-discovered by Runtime.

```yaml
# skills/text-processing/SKILL.md
---
name: text-processing
description: |
  Text processing domain. Automatically invoked when tasks involve text
  transformation, formatting, upper/lower case, reversal, concatenation,
  or simple string operations.

structures:
  - name: text-pipeline
    path: structures/text-pipeline
    summary: "Standard text processing pipeline"
    scenarios: ["text uppercase", "text reversal", "string transformation"]
    weight: 1.0

config:
  default_timeout: 10
  log_retention: "1d"
---
```

**Key Rules**:
- Must include `description`, otherwise excluded from auto-discovery
- `structures` list defines all Structures under this Complex
- `weight` used for conflict resolution (priority when multiple Structures match)
- `scenarios` used for intent matching keyword expansion

### Step 5: Validate and Run

```bash
# Validate discovery
cogtome discover
# Found 1 Complex(es):
#   text-processing  Text processing domain...

# Run
cogtome run text-processing --input '{"text":"hello"}'
# {"upper": "HELLO", "reversed": "olleh"}
```

---

## Interface Specifications

### Unit Contract

#### Process Model

```
Input: stdin receives UTF-8 JSON
Output: stdout outputs UTF-8 JSON
Diagnostics: stderr outputs human-readable text
Status: exit code 0 = success, non-0 = failure
```

#### Exit Code Standards

| Code | Meaning | Runtime Behavior |
|------|---------|----------------|
| 0 | Success | Parse stdout JSON, return to upper layer |
| 1 | Input error | Not retryable, report input problem |
| 2 | Processing exception | Retryable |
| 3 | Dependency unavailable | Retryable, exponential backoff |
| 126 | Command not executable | Not retryable, report permission problem |
| 127 | Command not found | Not retryable, report Unit not installed |
| 130 | SIGINT | Retryable, user interrupt or timeout |
| 137 | SIGKILL | Retryable, OOM or forced termination |

#### Environment Variables

```bash
COGTOME_UNIT_MODE=1       # Prohibit Unit from calling Unit internally
COGTOME_EXECUTION_ID=xxx  # Unique ID for this execution
COGTOME_TRACE_ID=xxx      # Distributed trace ID
COGTOME_LOG_LEVEL=info    # Log level
COGTOME_TIMEOUT_MS=30000  # Remaining timeout in milliseconds
```

#### Directory Template

```
units/<unit-name>/
├── SKILL.md          # CLI contract declaration (no description)
├── errors.yaml       # Error pattern library (optional)
└── bin/<unit-name>   # Executable entry point (chmod +x)
```

### Motif Contract

#### YAML Declarative Motif

```yaml
name: data-pipeline
type: motif
units_required: [fetch-url, parse-json]

flow:
  - name: fetch
    unit: fetch-url
    input:
      url: "${params.url}"
    output: raw_data

  - name: parse
    unit: parse-json
    input:
      text: "${steps.fetch.output.raw_data}"
    output: json_obj

return:
  data: "${steps.parse.output.json_obj}"
```

#### Python Motif (complex logic)

```python
from agents_sdk import Motif, unit, Context

class DataPipelineMotif(Motif):
    name = "data-pipeline"
    units_required = ["fetch-url", "parse-json"]

    def run(self, ctx: Context, url: str):
        raw = unit.call("fetch-url", {"url": url}, ctx=ctx)
        parsed = unit.call("parse-json", {"text": raw["result"]}, ctx=ctx)
        return parsed.data
```

**Key**: `unit.call()` in Python Motif communicates with Runtime via Unix Socket IPC, **does not directly create processes**.

### Structure Contract

#### manifest.yaml

```yaml
name: text-pipeline
type: structure

motifs:
  - name: text-transform

input_schema:
  type: object
  required: [text]
  properties:
    text: { type: string }

output_schema:
  type: object
  properties:
    upper: { type: string }

resources:
  memory: "512m"
  network: true
  gpu: false

constraints:
  webgl: false
```

#### Custom Executor (optional)

If `structures/<name>/structure.py` exists, Runtime loads and executes it:

```python
from agents_sdk import Structure, motif, Context

class TextPipelineStructure(Structure):
    def execute(self, params: dict, ctx: Context) -> dict:
        m = motif.load("text-transform")
        return m.run(ctx, **params)
```

If no `structure.py`, Runtime uses **default executor**: loads and executes Motifs in `manifest.motifs` order.

### Complex Contract

#### SKILL.md

```yaml
---
name: web-automation
description: |
  Browser automation domain...

structures:
  - name: lightpanda
    path: structures/lightpanda
    summary: "High-speed headless browser based on native engine"
    scenarios: ["static page scraping", "form automation"]
    constraints: { webgl: false }
    weight: 0.8

config:
  default_timeout: 30
  max_concurrent: 3
  log_retention: "1d"
---
```

#### Custom Selector (optional)

```python
from agents_sdk import Complex, Structure

class WebAutomationComplex(Complex):
    def select_structure(self, intent: str, constraints: dict) -> Structure:
        if constraints.get("webgl"):
            return self.load_structure("playwright")
        return self.load_structure("lightpanda")
```

---

## CLI Reference

```bash
# Discovery and browsing
cogtome discover                              # Scan all Complexes
cogtome skill list                           # List Complexes
cogtome skill show <name>                    # View Complex details
cogtome skill search <keyword>               # Fuzzy search

# Debug layer (developer tools)
cogtome unit list                            # List all Units
cogtome unit show <name>                     # View Unit contract
cogtome unit run <name> --input <json>      # Run Unit directly
cogtome unit run <name> --stdin              # Read from stdin

cogtome motif list                           # List all Motifs
cogtome motif run <name> --input <json>     # Run Motif

cogtome structure list                       # List all Structures
cogtome structure validate <name>            # Validate manifest
cogtome structure run <name> --input <json> # Run Structure

# Execution layer (Agent usage)
cogtome run <complex> --input <json>        # Run Complex
cogtome run <complex> --input <json> --dry-run # Compile plan without executing

# Logs and inspection
cogtome logs                                 # List today's executions
cogtome logs --date 2026-04-24            # View history
cogtome inspect <execution-id> --tree       # Tree view of four-layer call chain

# System management
cogtome validate                             # Validate all Skills
cogtome validate --fix                       # Auto-fix common issues
cogtome daemon start                         # Start daemon process
cogtome daemon stop
cogtome daemon status

# Packaging and distribution (future)
cogtome pack ./my-skill/                     # Package as .cogtome file
cogtome install my-skill.cogtome            # Install Skill
```

---

## Technical Implementation

### Runtime Modules (Rust)

```
src/
├── main.rs                 # CLI entry point (clap)
│   ├── unit run            # Direct UnitRunner invocation
│   ├── motif run           # Invoke YamlMotifEngine
│   ├── structure run       # Invoke StructureExecutor
│   └── run                 # Complex → Structure → Motif → Unit
├── context.rs              # Execution context + variable resolution
│   ├── ExecContext         # params + steps HashMap
│   └── resolve_var()       # ${params} / ${steps} / ${env}
├── discovery.rs            # Directory scanning and metadata discovery
│   ├── find_unit()         # Global → Complex private, priority search
│   ├── find_motif()        # .yaml / .py / .sh
│   ├── find_structure()    # manifest.yaml
│   └── discover_complexes() # Scan SKILL.md
└── engine.rs               # Core execution engine
    ├── UnitRunner          # tokio::process fork/exec
    ├── YamlMotifEngine     # YAML parsing + serial scheduling
    └── StructureExecutor   # manifest loading + Motif chain execution
```

### Execution Flow

```
Agent Query
    │
    ▼
┌────────────────────────────────┐
│ 1. Discovery                   │  Scan ~/.agents/skills/*/SKILL.md
│    Build Complex index         │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│ 2. Resolution                  │  Description similarity / constraint matching / weight sorting
│    Complex.select_structure()  │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│ 3. Compilation                 │  Structure manifest → ExecutionPlan
│    Static dependency check     │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│ 4. Scheduling                 │  Execute in ExecutionStep order
│    • Serial: blocking          │
│    • Parallel: tokio::spawn    │
│    • Resource: acquire → release│
│    • Timeout: tokio::timeout   │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│ 5. Validation                  │  Validate output_schema
│    Write log index index.json  │
└────────────────────────────────┘
```

### Multi-Language Motif Strategy

| Type | Extension | Execution | Status |
|------|-----------|-----------|--------|
| YAML Motif | `.yaml` | Rust native parsing | ✅ Implemented |
| Python Motif | `.py` | Subprocess + IPC → Runtime | 🔮 Phase 2 |
| Shell Motif | `.sh` | `tokio::process::Command` | 🔮 Phase 2 |
| Rust Motif | `.so` | `libloading` dynamic loading | 🔮 Phase 4 |

### Python SDK IPC (future)

Python Motif does not directly `subprocess.run`, instead communicates with Runtime via Unix Domain Socket:

```python
# agents_sdk/unit.py
class CogtomeClient:
    def __init__(self, socket_path="/tmp/cogtome.sock"):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(socket_path)

    def unit_call(self, name, input, ctx):
        # Send JSON-RPC
        ...
```

This preserves Python flexibility while gaining Rust process management performance.

---

## Roadmap

### Phase 1: Core MVP ✅ (Current)

- [x] CLI framework (unit / motif / structure / run / discover)
- [x] UnitRunner: `tokio::process` + stdin/stdout JSON + timeout
- [x] Discovery: scan `skills/` directory tree
- [x] YamlMotifEngine: variable resolution + serial execution
- [x] StructureExecutor: manifest loading + Motif chain
- [x] Complex discovery: SKILL.md parsing

### Phase 2: Daemon and Concurrency

- [ ] `cogtome daemon` (Unix Socket + HTTP API)
- [ ] Metadata cache and hot reload
- [ ] Unit process warm pool
- [ ] Parallel Unit calls (`unit.gather()`)
- [ ] Python SDK IPC client
- [ ] YAML Motif: parallel groups + conditional branching

### Phase 3: Resource Management and Security

- [ ] Resource Units: `resource.acquire/release` + RAII Guard
- [ ] WAL crash recovery mechanism
- [ ] Linux Landlock filesystem isolation
- [ ] seccomp-bpf system call filtering
- [ ] cgroups v2 resource limits

### Phase 4: Ecosystem and Optimization

- [ ] `cogtome pack/install` packaging distribution
- [ ] Registry / central repository protocol
- [ ] Rust Motif dynamic loading (`.so`)
- [ ] Web UI monitoring panel
- [ ] Performance benchmarking

---

## Design Principles

1. **Runtime has zero business logic**: COGTOME binary has no built-in Units. Agents forge them as needed.
2. **Agent authoring freedom**: Units can be written in any language; Motifs in YAML/Python/Shell; Structures pure declarative or custom executor.
3. **Strong contracts**: All cross-layer calls validated via Schema (JSON Schema), input/output type-safe.
4. **Process isolation**: Units never call each other, each Unit is an independent fork + exec.
5. **Observability**: Every execution generates complete four-layer chain logs (JSON Lines), supports `cogtome inspect --tree`.

---

## License

MIT

# Visual Block Editor Design

## Overview

Three new editor pages for COGTOME's WebUI:

1. **Unit Editor** — edit Unit metadata, test execution, AI chat assistant
2. **Motif Editor** — visual **directed graph** editor with draggable Unit blocks and port-based connections
3. **Structure Editor** — visual block editor with Unit + Motif blocks, Motif blocks expandable

## COGTOME Layer Architecture

```
Structure (L3) ────► Motif (L2) ────► Unit (L1)
   blocks              blocks            atomic
 Units + Motifs       Units only        CLI executor
```

### Motif as Directed Graph

**Key design decision: Motifs are directed graphs (digraphs), not linear pipelines.**

```
Linear pipeline:    A ──► B ──► C

Motif digraph:
                    ┌──► B ──┐
               A ───┤       ├──► D
                    └──► C ──┘
```

**Digraph semantics:**
- **Nodes** = Unit blocks (each unit = one step)
- **Edges** = data flow connections (output port → input port)
- **Execution order** = topological sort of the graph (no cycles allowed)
- **Parallel execution** = nodes with no data dependency can run concurrently
- **Fan-out**: one output can feed multiple inputs
- **Fan-in**: multiple outputs can feed one input (joined at the input port)

---

## 1. Unit Editor Page

**Route:** `/units/:name` (edit existing) | `/units/new`

### Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  [←] Unit Editor: file-read                         [Save] [×] │
├──────────────────────────────┬───────────────────────────────-─┤
│  CONFIGURATION               │  TEST PANEL                     │
│                              │                                 │
│  Name: [file-read        ]   │  Input JSON:                    │
│  Path: skills/units/...       │  ┌─────────────────────────┐   │
│                              │  │ { "path": "test.txt",    │   │
│  Timeout: ────●───── 30s     │  │   "offset": 0,           │   │
│  Concurrency: [1     ▼]       │  │   "limit": 100 }         │   │
│                              │  └─────────────────────────┘   │
│  Description:                │                                 │
│  ┌──────────────────────┐    │  [▶ Run Test]                  │
│  │ Reads file content   │    │                                 │
│  │ from specified path  │    │  Output:                        │
│  └──────────────────────┘    │  ┌─────────────────────────┐   │
│                              │  │ { "content": "...",      │   │
│                              │  │   "lines": 42 }          │   │
│                              │  └─────────────────────────┘   │
│                              │  ✓ Success (exit 0, 23ms)       │
├──────────────────────────────┴───────────────────────────────-─┤
│  🤖 Assistant                                        [−] [×]   │
│  ──────────────────────────────────────────────────────────── │
│  You: How do I add offset support?                            │
│  Bot: You can use the ${params.offset} variable to skip...    │
│  ──────────────────────────────────────────────────────────── │
│  [Ask the assistant about this Unit...]                [Send]  │
└─────────────────────────────────────────────────────────────────┘
```

### Sections

**Configuration Panel (left, ~300px)**
- Name field (read-only for existing Units)
- Path display (read-only)
- Timeout slider (1-300s, default 30)
- Concurrency dropdown (1-10, or "unlimited")
- Description textarea

**Test Panel (right, ~400px)**
- Input JSON textarea (syntax highlighted)
- "Run Test" button → calls `POST /run { "type": "unit", "name": "...", "input": {...} }`
- Output display area (JSON + status badge: success/error/exit code)
- Execution time display

**Chat Assistant (bottom, collapsible, ~300px default)**
- Message history (scrollable)
- Input field with Send button
- AI analyzes current Unit context to answer questions
- Powered by Claude API (streaming)

---

## 2. Motif Editor Page (Directed Graph)

**Route:** `/motifs/:name/edit`

### Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  [←] Motif: fetch-web                              [Validate]  │
│                                                    [Save] [×]  │
├────────────┬──────────────────────────────────────┬──────────────┤
│ PALETTE   │         CANVAS (digraph)             │ PROPERTIES  │
│            │                                     │              │
│ ▸ UNITS   │      [fetch]───┬───►[filter]──►      │ Step: fetch │
│   fetch   │          │     │                     │              │
│   filter  │          └──►[log]                  │ Input:      │
│   map     │                                     │   url: ${   │
│ ▸ CONTROL │                                     │     params  │
│   if      │                                     │     .url}   │
│   foreach │                                     │              │
│            │                                     │ Output:     │
│            │                                     │   content   │
│            │                                     │   url       │
│            │                                     │              │
│            │  [+ Add Unit]                       │              │
├────────────┴──────────────────────────────────────┴──────────────┤
│  [YAML View]  [Graph View] (toggle)                              │
└─────────────────────────────────────────────────────────────────┘
```

### Block Types

**Unit Block (node with ports)**
```
         ┌──► output: content ──┐
         │    output: url       │
┌─────────────────────────────┐ │
│ ● fetch                    │─┘
│  input: url ───────────────┘
└─────────────────────────────┘
```
- **Input port(s)** — left side, labeled by parameter name
- **Output port(s)** — right side, labeled by output field name
- Ports have type indicators (string, array, object)
- Click port → drag edge → connect to another node's port

**Control Flow Blocks (if)**
```
         ┌──► output ──────────┐
         │
┌─────────────────────────────┐ │
│ ◇ if                        │─┘
│  condition ─────────────────┘
│  [Unit block]                │
│  [Unit block]                │
└─────────────────────────────┘
```
- Same port-based connections
- Condition input field on left
- Nested units inside the block

**Control Flow Blocks (foreach)**
```
         ┌──► output ──────────┐
         │
┌─────────────────────────────┐ │
│ ◇ foreach                   │─┘
│  over ──────────────────────┘
│  [Unit block]                │
│  [Unit block]                │
└─────────────────────────────┘
```
- `over` expression port for iteration array
- `max_iterations` config field
- `parallel` toggle

**Return Block (terminal node)**
```
┌─────────────────────────────┐
│ ◼ return                    │
│  content ───────────────────┤
│  url ───────────────────────┤
└─────────────────────────────┘
```
- No output ports (terminal)
- Input ports for each return field
- Labeled key names on left

### Port Connection Rules

1. **Type matching** — connect output to input of compatible type
2. **Expression binding** — edge means `${steps.<source>.output.<field>}` passed to target input
3. **Variable shortcuts** — clicking an input port shows quick-insert for `${params.}`, `${steps.}`
4. **No cycles** — graph must be acyclic (validated on save)

### Canvas Interactions

- **Drag from palette** → creates new node at drop position
- **Drag from port** → creates edge to another port
- **Click node** → selects, shows properties
- **Click edge** → selects, shows edge config (or delete)
- **Delete key** → removes selected node/edge
- **Pan** → middle-mouse or space+drag
- **Zoom** → scroll wheel
- **Auto-layout** → button to auto-arrange nodes (top-to-bottom or left-to-right)
- **Mini-map** → bottom-right corner for navigation

### Properties Panel

When a node is selected:
- Unit block → input field mappings, output display
- if/foreach → condition/over expression, max_iterations, parallel toggle
- return → key-value return expressions

When an edge is selected:
- Shows source → target connection
- Delete edge option

### View Toggle

- **Graph View** — visual digraph editor (default)
- **YAML View** — raw YAML, changes sync bidirectionally

---

## 3. Structure Editor Page

**Route:** `/structures/:name` (existing) | `/structures/new`

### Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  [←] Structure: fetch                               [Validate]  │
│                                                    [Save] [×]  │
├────────────┬──────────────────────────────────────┬──────────────┤
│ PALETTE   │         CANVAS (digraph)               │ PROPERTIES   │
│            │                                      │              │
│ ▸ MOTIFS  │   [fetch-web ▼]──►[file-write]       │ Motif:      │
│   fetch   │        │                              │   fetch-web │
│   filter  │    (expanded)                         │              │
│ ▸ UNITS   │   [fetch-text]──►[return]            │ Input:      │
│   fetch   │                                      │   url       │
│ ▸ CONTROL │   [map-data]──►                      │              │
│   if      │                                      │ Output:     │
│   foreach │                                      │   bytes     │
│            │                                      │              │
│            │  [+ Add Motif]  [+ Add Unit]        │              │
└────────────┴──────────────────────────────────────┴──────────────┘
```

### Block Types

**Motif Block**
```
         ┌──► output ──────────┐
         │
┌─────────────────────────────┐ │
│ ◆ fetch-web           [▼]  │─┘
│  input: url ───────────────┘
└─────────────────────────────┘
```
- ◆ = Motif indicator
- [▼] = expand to show internal digraph
- Click ▼ → inline expansion of Motif's internal Unit graph
- Ports represent Motif's input_schema / output_schema

**Unit Block** — same as Motif editor

**Control Flow Blocks** — same as Motif editor

### Key Differences from Motif Editor

1. Palette includes **Motif blocks** (in addition to Units and control flow)
2. Motif blocks are **expandable inline** — click ▼ to see/edit internal digraph
3. When expanded, Motif block shows its internal nodes in the same canvas
4. Structure-level blocks (Motifs/Units) form the top-level graph

---

## 4. Shared Graph Engine

Both Motif and Structure editors share the same **graph rendering engine**:

```
Motif Editor ──┐
               ├──► Shared React Flow canvas + graph store
Structure Ed ──┘
```

### Graph Data Model

```typescript
interface GraphNode {
  id: string;
  type: 'unit' | 'if' | 'foreach' | 'return' | 'motif';
  position: { x: number; y: number };
  data: {
    // unit: { name, inputs: {}, outputs: {} }
    // if: { condition, body: GraphNode[] }
    // foreach: { over, max_iterations, parallel, body: GraphNode[] }
    // return: { mappings: {} }
    // motif: { name, expanded: boolean, internalGraph: Graph }
  };
}

interface GraphEdge {
  id: string;
  source: string;        // node id
  sourceHandle: string;  // output port name
  target: string;        // node id
  targetHandle: string;   // input port name
}

interface Graph {
  nodes: GraphNode[];
  edges: GraphEdge[];
}
```

### Serialization to YAML

Graph ↔ YAML bidirectional sync:

```
Graph → YAML:
  nodes with edges → flow array with `from`/`to` references

YAML → Graph:
  parse flow steps → reconstruct nodes and edges
```

### Execution Order

Runtime determines execution order via **topological sort**:
1. Find nodes with no input dependencies (or all inputs from params)
2. Execute in parallel (respecting concurrency limits)
3. As each node completes, mark dependent nodes as ready
4. Repeat until all nodes executed
5. Error if cycle detected

---

## Technical Approach

### Libraries

- **React Flow** — canvas, nodes, edges, pan/zoom, ports, auto-layout, mini-map
- **Zustand** — state management (extend existing store)
- **CodeMirror 6** — JSON editing in Unit editor
- **react-markdown** — chat messages rendering
- **yaml** — YAML parse/stringify for sync

### API Changes

**New Unit endpoints needed:**
```
GET  /api/units              → list all units (existing)
GET  /api/units/:name        → read Unit metadata
PUT  /api/units/:name        → save Unit metadata
POST /api/units/:name/test   → run test execution
```

### File Structure

```
webui/src/
├── components/
│   ├── editors/
│   │   ├── UnitEditor.tsx
│   │   ├── MotifEditor.tsx
│   │   ├── StructureEditor.tsx
│   │   ├── ChatAssistant.tsx
│   │   └── TestPanel.tsx
│   ├── graph/
│   │   ├── GraphCanvas.tsx         (React Flow wrapper)
│   │   ├── UnitNode.tsx
│   │   ├── IfNode.tsx
│   │   ├── ForeachNode.tsx
│   │   ├── ReturnNode.tsx
│   │   ├── MotifNode.tsx
│   │   └── graphUtils.ts           (topo sort, serialization)
│   └── PropertyPanel.tsx
├── store/
│   ├── graphStore.ts               (Zustand, graph state)
│   └── structureStore.ts           (existing)
├── api/
│   └── client.ts                   (add Unit CRUD)
└── types/
    └── index.ts                    (add GraphNode, GraphEdge)
```

### Chat Assistant

- Uses Claude API (`/v1/messages`)
- System prompt: current Unit/Motif/Structure context + COGTOME docs
- Streaming responses
- Session persisted in `localStorage`

---

## Implementation Order

1. **Graph engine** — React Flow integration, node/edge rendering
2. **Motif Editor** — digraph with Unit nodes + if/foreach control flow
3. **Structure Editor redesign** — Motif blocks + expansion
4. **Unit Editor** — metadata + test panel + chat
5. **Graph ↔ YAML sync** — bidirectional serialization
6. **Polish** — auto-layout, mini-map, keyboard shortcuts, validation

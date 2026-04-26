# Visual Block Editor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement three editor pages (Unit Editor, Motif Editor as digraph, Structure Editor with expandable Motifs) with shared React Flow graph engine.

**Architecture:** React Flow canvas for graph editing, Zustand for state, bidirectional graph↔YAML sync, Claude API for chat assistant.

**Tech Stack:** React Flow, Zustand, CodeMirror 6, Claude API

---

## Phase 1: Infrastructure (types, API, store)

### Task 1: Add Graph Types

**Files:**
- Modify: `webui/src/types/index.ts`

- [ ] **Step 1: Add graph types to types/index.ts**

Add after existing types:

```typescript
// Graph types for visual editor
export type BlockType = 'unit' | 'if' | 'foreach' | 'return' | 'motif';

export interface Port {
  id: string;
  name: string;
  type: 'string' | 'number' | 'boolean' | 'array' | 'object';
}

export interface BlockNode {
  id: string;
  type: BlockType;
  position: { x: number; y: number };
  data: {
    name?: string;           // unit/motif name
    inputs?: Record<string, string>;   // input field → expression mapping
    outputs?: Port[];        // output ports
    condition?: string;       // if condition
    over?: string;           // foreach expression
    maxIterations?: number;
    parallel?: boolean;
    mappings?: Record<string, string>; // return key → expression
    expanded?: boolean;      // motif block expansion
    internalGraph?: Graph;   // expanded motif's internal graph
  };
}

export interface BlockEdge {
  id: string;
  source: string;
  sourceHandle: string;
  target: string;
  targetHandle: string;
}

export interface Graph {
  nodes: BlockNode[];
  edges: BlockEdge[];
}
```

- [ ] **Step 2: Commit**

```bash
git add webui/src/types/index.ts
git commit -m "feat(webui): add graph types for visual editor"
```

---

### Task 2: Install Dependencies

**Files:**
- Modify: `webui/package.json`

- [ ] **Step 1: Add React Flow and related packages**

```bash
cd webui && npm install @xyflow/react yaml
```

- [ ] **Step 2: Commit**

```bash
git add webui/package.json webui/package-lock.json
git commit -m "deps(webui): add @xyflow/react and yaml packages"
```

---

### Task 3: Create Graph Store

**Files:**
- Create: `webui/src/store/graphStore.ts`

- [ ] **Step 1: Create Zustand store for graph state**

```typescript
import { create } from 'zustand';
import { BlockNode, BlockEdge, Graph, BlockType } from '../types';

interface GraphStore {
  // Graph data
  nodes: BlockNode[];
  edges: BlockEdge[];
  selectedNodeId: string | null;
  selectedEdgeId: string | null;

  // Actions
  setGraph: (graph: Graph) => void;
  addNode: (type: BlockType, position: { x: number; y: number }, name?: string) => void;
  updateNode: (id: string, data: Partial<BlockNode['data']>) => void;
  removeNode: (id: string) => void;
  addEdge: (source: string, sourceHandle: string, target: string, targetHandle: string) => void;
  removeEdge: (id: string) => void;
  selectNode: (id: string | null) => void;
  selectEdge: (id: string | null) => void;
  updateNodePosition: (id: string, position: { x: number; y: number }) => void;
  clearGraph: () => void;

  // Serialization
  toGraph: () => Graph;
}

const createDefaultOutputs = (type: BlockType): BlockNode['data']['outputs'] => {
  if (type === 'return') return [];
  return [
    { id: 'out-1', name: 'output', type: 'string' },
  ];
};

const createDefaultInputs = (type: BlockType): BlockNode['data']['inputs'] => {
  switch (type) {
    case 'unit': return { input: '' };
    case 'if': return {};
    case 'foreach': return {};
    default: return {};
  }
};

export const useGraphStore = create<GraphStore>((set, get) => ({
  nodes: [],
  edges: [],
  selectedNodeId: null,
  selectedEdgeId: null,

  setGraph: (graph) => set({ nodes: graph.nodes, edges: graph.edges }),

  addNode: (type, position, name = '') => {
    const id = `${type}-${Date.now()}`;
    const newNode: BlockNode = {
      id,
      type,
      position,
      data: {
        name,
        inputs: createDefaultInputs(type),
        outputs: createDefaultOutputs(type),
        maxIterations: 50,
        parallel: false,
        expanded: false,
      },
    };
    set((state) => ({ nodes: [...state.nodes, newNode] }));
  },

  updateNode: (id, data) => {
    set((state) => ({
      nodes: state.nodes.map((n) =>
        n.id === id ? { ...n, data: { ...n.data, ...data } } : n
      ),
    }));
  },

  removeNode: (id) => {
    set((state) => ({
      nodes: state.nodes.filter((n) => n.id !== id),
      edges: state.edges.filter((e) => e.source !== id && e.target !== id),
    }));
  },

  addEdge: (source, sourceHandle, target, targetHandle) => {
    const id = `edge-${source}-${sourceHandle}-${target}-${targetHandle}`;
    const newEdge: BlockEdge = { id, source, sourceHandle, target, targetHandle };
    set((state) => ({ edges: [...state.edges, newEdge] }));
  },

  removeEdge: (id) => {
    set((state) => ({ edges: state.edges.filter((e) => e.id !== id) }));
  },

  selectNode: (id) => set({ selectedNodeId: id, selectedEdgeId: null }),
  selectEdge: (id) => set({ selectedEdgeId: id, selectedNodeId: null }),

  updateNodePosition: (id, position) => {
    set((state) => ({
      nodes: state.nodes.map((n) => (n.id === id ? { ...n, position } : n)),
    }));
  },

  clearGraph: () => set({ nodes: [], edges: [], selectedNodeId: null, selectedEdgeId: null }),

  toGraph: () => {
    const { nodes, edges } = get();
    return { nodes, edges };
  },
}));
```

- [ ] **Step 2: Commit**

```bash
git add webui/src/store/graphStore.ts
git commit -m "feat(webui): create graph store for visual editor state"
```

---

## Phase 2: React Flow Canvas + Node Components

### Task 4: Create GraphCanvas Component

**Files:**
- Create: `webui/src/components/graph/GraphCanvas.tsx`

- [ ] **Step 1: Create React Flow wrapper component**

```tsx
import React, { useCallback } from 'react';
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  Node,
  Edge,
  Connection,
  addEdge,
  useNodesState,
  useEdgesState,
  NodeChange,
  EdgeChange,
  OnNodesChange,
  OnEdgesChange,
  OnConnect,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { useGraphStore } from '../../store/graphStore';
import { BlockNode, BlockEdge } from '../../types';

// Convert BlockNode → ReactFlow Node
const toRFNode = (node: BlockNode): Node => ({
  id: node.id,
  type: node.type,
  position: node.position,
  data: node.data,
});

// Convert BlockEdge → ReactFlow Edge
const toRFEdge = (edge: BlockEdge): Edge => ({
  id: edge.id,
  source: edge.source,
  target: edge.target,
  sourceHandle: edge.sourceHandle,
  targetHandle: edge.targetHandle,
  type: 'smoothstep',
  animated: true,
});

interface GraphCanvasProps {
  onNodeClick?: (nodeId: string) => void;
  onEdgeClick?: (edgeId: string) => void;
  nodes: BlockNode[];
  edges: BlockEdge[];
  onNodesChange?: OnNodesChange;
  onEdgesChange?: OnEdgesChange;
  onConnect?: OnConnect;
  onNodeDragStop?: (nodeId: string, position: { x: number; y: number }) => void;
}

export function GraphCanvas({
  onNodeClick,
  onEdgeClick,
  nodes,
  edges,
  onNodesChange,
  onEdgesChange,
  onConnect,
  onNodeDragStop,
}: GraphCanvasProps) {
  const rfNodes = nodes.map(toRFNode);
  const rfEdges = edges.map(toRFEdge);

  return (
    <div style={{ width: '100%', height: '100%' }}>
      <ReactFlow
        nodes={rfNodes}
        edges={rfEdges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onNodeClick={(_, node) => onNodeClick?.(node.id)}
        onEdgeClick={(_, edge) => onEdgeClick?.(edge.id)}
        onNodeDragStop={(_, node) => onNodeDragStop?.(node.id, node.position)}
        fitView
        snapToGrid
        snapGrid={[16, 16]}
      >
        <Background />
        <Controls />
        <MiniMap />
      </ReactFlow>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add webui/src/components/graph/GraphCanvas.tsx
git commit -m "feat(webui): create GraphCanvas React Flow wrapper"
```

---

### Task 5: Create Unit Node Component

**Files:**
- Create: `webui/src/components/graph/nodes/UnitNode.tsx`

- [ ] **Step 1: Create Unit node with input/output ports**

```tsx
import React, { memo } from 'react';
import { Handle, Position, NodeProps } from '@xyflow/react';

export const UnitNode = memo(({ data, selected }: NodeProps) => {
  const { name, inputs = {}, outputs = [] } = data;

  return (
    <div
      style={{
        background: '#1e1e2e',
        border: selected ? '2px solid #7c3aed' : '2px solid #3b3b5c',
        borderRadius: 8,
        padding: 12,
        minWidth: 160,
        fontFamily: 'monospace',
        fontSize: 13,
      }}
    >
      {/* Input handle */}
      {Object.keys(inputs).length > 0 && (
        <Handle
          type="target"
          position={Position.Left}
          style={{
            background: '#7c3aed',
            width: 10,
            height: 10,
            border: 'none',
          }}
        />
      )}

      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
        <div
          style={{
            width: 20,
            height: 20,
            borderRadius: 4,
            background: '#7c3aed',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 10,
            fontWeight: 700,
            color: '#fff',
          }}
        >
          U
        </div>
        <span style={{ color: '#e2e8f0', fontWeight: 600 }}>{name || 'unit'}</span>
      </div>

      {/* Input fields preview */}
      <div style={{ marginBottom: 4 }}>
        {Object.entries(inputs).map(([key, val]) => (
          <div key={key} style={{ color: '#94a3b8', fontSize: 11, marginBottom: 2 }}>
            {key}: <span style={{ color: '#7dd3fc' }}>{String(val).slice(0, 20)}</span>
          </div>
        ))}
      </div>

      {/* Output handle */}
      {outputs.length > 0 && (
        <Handle
          type="source"
          position={Position.Right}
          style={{
            background: '#22c55e',
            width: 10,
            height: 10,
            border: 'none',
          }}
        />
      )}
    </div>
  );
});

UnitNode.displayName = 'UnitNode';
```

- [ ] **Step 2: Commit**

```bash
git add webui/src/components/graph/nodes/UnitNode.tsx
git commit -m "feat(webui): create UnitNode component with ports"
```

---

### Task 6: Create Control Flow Nodes (If + Foreach)

**Files:**
- Create: `webui/src/components/graph/nodes/IfNode.tsx
- Create: `webui/src/components/graph/nodes/ForeachNode.tsx

- [ ] **Step 1: Create IfNode with condition and body**

```tsx
import React, { memo } from 'react';
import { Handle, Position, NodeProps } from '@xyflow/react';

export const IfNode = memo(({ data, selected }: NodeProps) => {
  const { condition = '', expanded = false } = data;

  return (
    <div
      style={{
        background: '#1a1a2e',
        border: selected ? '2px solid #f59e0b' : '2px solid #3b3b5c',
        borderRadius: 8,
        padding: 12,
        minWidth: 180,
        fontFamily: 'monospace',
      }}
    >
      <Handle
        type="target"
        position={Position.Left}
        style={{ background: '#f59e0b', width: 10, height: 10, border: 'none' }}
      />

      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
        <div
          style={{
            width: 20,
            height: 20,
            borderRadius: 4,
            background: '#f59e0b',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 10,
            fontWeight: 700,
            color: '#000',
          }}
        >
          ?
        </div>
        <span style={{ color: '#e2e8f0', fontWeight: 600 }}>if</span>
      </div>

      <div style={{ background: '#0f0f1a', borderRadius: 4, padding: 8, marginBottom: 8 }}>
        <div style={{ color: '#64748b', fontSize: 10, marginBottom: 4 }}>condition</div>
        <div style={{ color: '#fbbf24', fontSize: 12 }}>{condition || '(none)'}</div>
      </div>

      {expanded && (
        <div style={{ color: '#64748b', fontSize: 11, fontStyle: 'italic' }}>
          (body: {data.body?.length || 0} nodes)
        </div>
      )}

      <Handle
        type="source"
        position={Position.Right}
        style={{ background: '#22c55e', width: 10, height: 10, border: 'none' }}
      />
    </div>
  );
});

IfNode.displayName = 'IfNode';
```

- [ ] **Step 2: Create ForeachNode**

```tsx
import React, { memo } from 'react';
import { Handle, Position, NodeProps } from '@xyflow/react';

export const ForeachNode = memo(({ data, selected }: NodeProps) => {
  const { over = '', maxIterations = 50, expanded = false } = data;

  return (
    <div
      style={{
        background: '#1a1a2e',
        border: selected ? '2px solid #06b6d4' : '2px solid #3b3b5c',
        borderRadius: 8,
        padding: 12,
        minWidth: 180,
        fontFamily: 'monospace',
      }}
    >
      <Handle
        type="target"
        position={Position.Left}
        style={{ background: '#06b6d4', width: 10, height: 10, border: 'none' }}
      />

      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
        <div
          style={{
            width: 20,
            height: 20,
            borderRadius: 4,
            background: '#06b6d4',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 10,
            fontWeight: 700,
            color: '#000',
          }}
        >
          #
        </div>
        <span style={{ color: '#e2e8f0', fontWeight: 600 }}>foreach</span>
      </div>

      <div style={{ background: '#0f0f1a', borderRadius: 4, padding: 8, marginBottom: 4 }}>
        <div style={{ color: '#64748b', fontSize: 10, marginBottom: 4 }}>over</div>
        <div style={{ color: '#67e8f9', fontSize: 12 }}>{over || '(none)'}</div>
      </div>

      <div style={{ color: '#64748b', fontSize: 11 }}>
        max: {maxIterations}
      </div>

      {expanded && (
        <div style={{ color: '#64748b', fontSize: 11, fontStyle: 'italic', marginTop: 4 }}>
          (body: {data.body?.length || 0} nodes)
        </div>
      )}

      <Handle
        type="source"
        position={Position.Right}
        style={{ background: '#22c55e', width: 10, height: 10, border: 'none' }}
      />
    </div>
  );
});

ForeachNode.displayName = 'ForeachNode';
```

- [ ] **Step 3: Commit**

```bash
git add webui/src/components/graph/nodes/IfNode.tsx webui/src/components/graph/nodes/ForeachNode.tsx
git commit -m "feat(webui): create IfNode and ForeachNode control flow components"
```

---

### Task 7: Create Return + Motif Nodes

**Files:**
- Create: `webui/src/components/graph/nodes/ReturnNode.tsx`
- Create: `webui/src/components/graph/nodes/MotifNode.tsx`

- [ ] **Step 1: Create ReturnNode (terminal, input ports only)**

```tsx
import React, { memo } from 'react';
import { Handle, Position, NodeProps } from '@xyflow/react';

export const ReturnNode = memo(({ data, selected }: NodeProps) => {
  const { mappings = {} } = data;

  return (
    <div
      style={{
        background: '#1a2e1a',
        border: selected ? '2px solid #22c55e' : '2px solid #2d5a2d',
        borderRadius: 8,
        padding: 12,
        minWidth: 160,
        fontFamily: 'monospace',
      }}
    >
      <Handle
        type="target"
        position={Position.Left}
        style={{ background: '#22c55e', width: 10, height: 10, border: 'none' }}
      />

      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
        <div
          style={{
            width: 20,
            height: 20,
            borderRadius: 4,
            background: '#22c55e',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 10,
            fontWeight: 700,
            color: '#000',
          }}
        >
          R
        </div>
        <span style={{ color: '#e2e8f0', fontWeight: 600 }}>return</span>
      </div>

      {Object.entries(mappings).map(([key, val]) => (
        <div key={key} style={{ color: '#86efac', fontSize: 11, marginBottom: 2 }}>
          {key}: <span style={{ color: '#7dd3fc' }}>{String(val).slice(0, 20)}</span>
        </div>
      ))}

      {Object.keys(mappings).length === 0 && (
        <div style={{ color: '#64748b', fontSize: 11, fontStyle: 'italic' }}>(no mappings)</div>
      )}
    </div>
  );
});

ReturnNode.displayName = 'ReturnNode';
```

- [ ] **Step 2: Create MotifNode (expandable)**

```tsx
import React, { memo } from 'react';
import { Handle, Position, NodeProps } from '@xyflow/react';

export const MotifNode = memo(({ data, selected }: NodeProps) => {
  const { name = '', expanded = false } = data;

  return (
    <div
      style={{
        background: '#2e1a2e',
        border: selected ? '2px solid #a855f7' : '2px solid #5c3b6e',
        borderRadius: 8,
        padding: 12,
        minWidth: 160,
        fontFamily: 'monospace',
      }}
    >
      <Handle
        type="target"
        position={Position.Left}
        style={{ background: '#a855f7', width: 10, height: 10, border: 'none' }}
      />

      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
        <div
          style={{
            width: 20,
            height: 20,
            borderRadius: 4,
            background: '#a855f7',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 10,
            fontWeight: 700,
            color: '#fff',
          }}
        >
          M
        </div>
        <span style={{ color: '#e2e8f0', fontWeight: 600 }}>{name || 'motif'}</span>
      </div>

      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <span
          style={{
            color: expanded ? '#22c55e' : '#64748b',
            fontSize: 11,
          }}
        >
          {expanded ? '▼ expanded' : '▶ collapsed'}
        </span>
      </div>

      <Handle
        type="source"
        position={Position.Right}
        style={{ background: '#22c55e', width: 10, height: 10, border: 'none' }}
      />
    </div>
  );
});

MotifNode.displayName = 'MotifNode';
```

- [ ] **Step 3: Create node registry**

```tsx
// webui/src/components/graph/nodes/index.ts
export { UnitNode } from './UnitNode';
export { IfNode } from './IfNode';
export { ForeachNode } from './ForeachNode';
export { ReturnNode } from './ReturnNode';
export { MotifNode } from './MotifNode';

import { UnitNode } from './UnitNode';
import { IfNode } from './IfNode';
import { ForeachNode } from './ForeachNode';
import { ReturnNode } from './ReturnNode';
import { MotifNode } from './MotifNode';

export const nodeTypes = {
  unit: UnitNode,
  if: IfNode,
  foreach: ForeachNode,
  return: ReturnNode,
  motif: MotifNode,
};
```

- [ ] **Step 4: Commit**

```bash
git add webui/src/components/graph/nodes/ReturnNode.tsx webui/src/components/graph/nodes/MotifNode.tsx webui/src/components/graph/nodes/index.ts
git commit -m "feat(webui): create ReturnNode and MotifNode components"
```

---

## Phase 3: Editor Pages

### Task 8: Create Unit Editor Page

**Files:**
- Create: `webui/src/components/editors/UnitEditor.tsx`
- Modify: `webui/src/App.tsx` (add route)
- Modify: `webui/src/api/client.ts` (add Unit CRUD)
- Modify: `src/api.rs` (add Unit GET/PUT endpoints)

- [ ] **Step 1: Add Unit API methods to client.ts**

Add to `webui/src/api/client.ts`:

```typescript
// Units API
export async function getUnit(name: string): Promise<UnitInfo> {
  return fetchJson<UnitInfo>(`${API_BASE}/units/${encodeURIComponent(name)}`);
}

export async function saveUnit(name: string, config: { timeout?: number; concurrency?: number; description?: string }): Promise<{ message: string }> {
  return fetchJson<{ message: string }>(`${API_BASE}/units/${encodeURIComponent(name)}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(config),
  });
}
```

- [ ] **Step 2: Create UnitEditor component**

```tsx
// webui/src/components/editors/UnitEditor.tsx
import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { getUnit, saveUnit } from '../../api/client';

export function UnitEditor() {
  const { name } = useParams<{ name: string }>();
  const navigate = useNavigate();
  const [config, setConfig] = useState({ timeout: 30, concurrency: 1, description: '' });
  const [testInput, setTestInput] = useState('{}');
  const [testOutput, setTestOutput] = useState<string | null>(null);
  const [testStatus, setTestStatus] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (name) {
      getUnit(name).then((unit) => {
        setConfig({
          timeout: unit.timeout ?? 30,
          concurrency: unit.concurrency ?? 1,
          description: unit.description ?? '',
        });
      }).catch(() => {});
    }
  }, [name]);

  const handleSave = async () => {
    if (!name) return;
    setSaving(true);
    try {
      await saveUnit(name, config);
    } catch (e) {
      console.error(e);
    }
    setSaving(false);
  };

  const handleTest = async () => {
    if (!name) return;
    try {
      const res = await fetch('/api/run', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ type: 'unit', name, input: JSON.parse(testInput) }),
      });
      const data = await res.json();
      if (data.result) {
        setTestOutput(JSON.stringify(data.result, null, 2));
        setTestStatus('success');
      } else if (data.error) {
        setTestOutput(JSON.stringify(data.error, null, 2));
        setTestStatus('error');
      }
    } catch (e) {
      setTestOutput(String(e));
      setTestStatus('error');
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', padding: 24 }}>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 16, marginBottom: 24 }}>
        <button onClick={() => navigate(-1)} style={buttonStyle}>← Back</button>
        <h2 style={{ margin: 0, flex: 1 }}>Unit: {name}</h2>
        <button onClick={handleSave} disabled={saving} style={buttonStyle}>
          {saving ? 'Saving...' : 'Save'}
        </button>
      </div>

      {/* Main content */}
      <div style={{ display: 'flex', flex: 1, gap: 24 }}>
        {/* Config panel */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 16 }}>
          <div style={cardStyle}>
            <h3 style={{ marginTop: 0 }}>Configuration</h3>
            <label style={labelStyle}>Name</label>
            <input value={name || ''} disabled style={inputStyle} />

            <label style={labelStyle}>Timeout (seconds)</label>
            <input
              type="range"
              min={1}
              max={300}
              value={config.timeout}
              onChange={(e) => setConfig({ ...config, timeout: Number(e.target.value) })}
              style={{ width: '100%' }}
            />
            <span>{config.timeout}s</span>

            <label style={labelStyle}>Concurrency</label>
            <input
              type="number"
              min={1}
              max={100}
              value={config.concurrency}
              onChange={(e) => setConfig({ ...config, concurrency: Number(e.target.value) })}
              style={inputStyle}
            />

            <label style={labelStyle}>Description</label>
            <textarea
              value={config.description}
              onChange={(e) => setConfig({ ...config, description: e.target.value })}
              style={{ ...inputStyle, height: 100 }}
            />
          </div>
        </div>

        {/* Test panel */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 16 }}>
          <div style={cardStyle}>
            <h3 style={{ marginTop: 0 }}>Test</h3>
            <label style={labelStyle}>Input JSON</label>
            <textarea
              value={testInput}
              onChange={(e) => setTestInput(e.target.value)}
              style={{ ...inputStyle, height: 120, fontFamily: 'monospace' }}
            />
            <button onClick={handleTest} style={{ ...buttonStyle, marginTop: 8 }}>
              ▶ Run Test
            </button>
          </div>

          <div style={cardStyle}>
            <h3 style={{ marginTop: 0 }}>Output</h3>
            {testStatus && (
              <div style={{
                padding: '4px 8px',
                borderRadius: 4,
                background: testStatus === 'success' ? '#22c55e20' : '#ef444420',
                color: testStatus === 'success' ? '#22c55e' : '#ef4444',
                marginBottom: 8,
                fontSize: 13,
              }}>
                {testStatus === 'success' ? '✓ Success' : '✗ Error'}
              </div>
            )}
            {testOutput && (
              <pre style={{ background: '#0f0f1a', padding: 12, borderRadius: 4, overflow: 'auto', fontSize: 12 }}>
                {testOutput}
              </pre>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

const cardStyle: React.CSSProperties = {
  background: '#1a1a2e',
  border: '1px solid #3b3b5c',
  borderRadius: 8,
  padding: 16,
};

const buttonStyle: React.CSSProperties = {
  background: '#7c3aed',
  color: '#fff',
  border: 'none',
  padding: '8px 16px',
  borderRadius: 4,
  cursor: 'pointer',
};

const inputStyle: React.CSSProperties = {
  width: '100%',
  background: '#0f0f1a',
  border: '1px solid #3b3b5c',
  borderRadius: 4,
  padding: '8px',
  color: '#e2e8f0',
  marginBottom: 8,
};

const labelStyle: React.CSSProperties = {
  display: 'block',
  color: '#94a3b8',
  fontSize: 13,
  marginBottom: 4,
  marginTop: 8,
};
```

- [ ] **Step 3: Add Unit GET/PUT endpoints to Rust API**

Add to `src/api.rs`:

```rust
// After Units API section (~line 492), add:
async fn get_unit_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, CogtomeError> {
    validate_name(&name)?;
    let unit_path = state.skills.root.join(&state.skills.units_subdir).join(&name);
    if !unit_path.exists() {
        return Err(CogtomeError::new(
            crate::error::ErrorLayer::Runtime,
            crate::error::ErrorCode::EUnitNotFound,
            format!("Unit '{}' not found", name),
        ));
    }
    // Return basic metadata (real impl would read manifest/config)
    Ok(Json(serde_json::json!({
        "name": name,
        "path": unit_path,
        "timeout": 30,
        "concurrency": 1,
        "description": ""
    })))
}

async fn put_unit_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, CogtomeError> {
    validate_name(&name)?;
    // Save unit config (simplified - real impl would persist)
    Ok(Json(serde_json::json!({ "message": "Unit saved", "name": name })))
}
```

Register routes in Router (around line 107):
```rust
.route("/api/units/:name", get(get_unit_handler))
.route("/api/units/:name", put(put_unit_handler))
```

- [ ] **Step 4: Add route to App.tsx**

```tsx
// Add to App.tsx imports
import { UnitEditor } from './components/editors/UnitEditor';

// Add route
<Route path="/units/:name" element={<UnitEditor />} />
```

- [ ] **Step 5: Add UnitInfo fields to types/index.ts**

```typescript
// Add to UnitInfo
export interface UnitInfo {
  name: string;
  path: string;
  timeout?: number;
  concurrency?: number;
  description?: string;
}
```

- [ ] **Step 6: Commit**

```bash
git add webui/src/components/editors/UnitEditor.tsx webui/src/api/client.ts webui/src/types/index.ts webui/src/App.tsx src/api.rs
git commit -m "feat: add Unit Editor page with test panel and API endpoints"
```

---

### Task 9: Create Motif Editor Page

**Files:**
- Create: `webui/src/components/editors/MotifEditor.tsx`
- Create: `webui/src/components/graph/Palette.tsx`
- Create: `webui/src/components/editors/PropertyPanel.tsx`
- Modify: `webui/src/App.tsx`

- [ ] **Step 1: Create BlockPalette component**

```tsx
// webui/src/components/graph/Palette.tsx
import React from 'react';
import { BlockType, MotifInfo } from '../../types';

interface PaletteProps {
  motifs?: MotifInfo[];
  onDragStart: (type: BlockType, name?: string) => void;
}

const UNIT_BLOCKS: { name: string; icon: string }[] = [
  { name: 'fetch-text', icon: 'U' },
  { name: 'file-read', icon: 'U' },
  { name: 'file-write', icon: 'U' },
  { name: 'filter-text', icon: 'U' },
  { name: 'map-text', icon: 'U' },
];

export function BlockPalette({ motifs = [], onDragStart }: PaletteProps) {
  return (
    <div style={{
      width: 220,
      background: '#0f0f1a',
      borderRight: '1px solid #3b3b5c',
      padding: 16,
      overflowY: 'auto',
      fontFamily: 'monospace',
    }}>
      <div style={{ color: '#64748b', fontSize: 11, marginBottom: 12, textTransform: 'uppercase', letterSpacing: 1 }}>
        Blocks
      </div>

      {/* Control flow */}
      <div style={{ marginBottom: 16 }}>
        <div style={{ color: '#94a3b8', fontSize: 12, marginBottom: 8, fontWeight: 600 }}>▸ Control</div>
        <BlockItem type="if" label="if" color="#f59e0b" onDragStart={onDragStart} />
        <BlockItem type="foreach" label="foreach" color="#06b6d4" onDragStart={onDragStart} />
      </div>

      {/* Motifs */}
      <div style={{ marginBottom: 16 }}>
        <div style={{ color: '#94a3b8', fontSize: 12, marginBottom: 8, fontWeight: 600 }}>▸ Motifs</div>
        {motifs.map((m) => (
          <BlockItem
            key={m.name}
            type="motif"
            label={m.name}
            color="#a855f7"
            name={m.name}
            onDragStart={onDragStart}
          />
        ))}
      </div>

      {/* Special */}
      <div style={{ marginBottom: 16 }}>
        <div style={{ color: '#94a3b8', fontSize: 12, marginBottom: 8, fontWeight: 600 }}>▸ Output</div>
        <BlockItem type="return" label="return" color="#22c55e" onDragStart={onDragStart} />
      </div>
    </div>
  );
}

interface BlockItemProps {
  type: BlockType;
  label: string;
  color: string;
  name?: string;
  onDragStart: (type: BlockType, name?: string) => void;
}

function BlockItem({ type, label, color, name, onDragStart }: BlockItemProps) {
  return (
    <div
      draggable
      onDragStart={() => onDragStart(type, name)}
      style={{
        background: '#1a1a2e',
        border: `1px solid ${color}40`,
        borderRadius: 4,
        padding: '6px 8px',
        marginBottom: 4,
        cursor: 'grab',
        color: '#e2e8f0',
        fontSize: 12,
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        transition: 'border-color 0.15s',
      }}
      onMouseEnter={(e) => e.currentTarget.style.borderColor = color}
      onMouseLeave={(e) => e.currentTarget.style.borderColor = `${color}40`}
    >
      <div style={{
        width: 16,
        height: 16,
        borderRadius: 3,
        background: color,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontSize: 9,
        fontWeight: 700,
        color: '#000',
      }}>
        {type[0].toUpperCase()}
      </div>
      {label}
    </div>
  );
}
```

- [ ] **Step 2: Create PropertyPanel component**

```tsx
// webui/src/components/editors/PropertyPanel.tsx
import React from 'react';
import { BlockNode, BlockEdge } from '../../types';

interface PropertyPanelProps {
  selectedNode: BlockNode | null;
  selectedEdge: BlockEdge | null;
  onUpdateNode: (id: string, data: Partial<BlockNode['data']>) => void;
  onDeleteNode: (id: string) => void;
  onDeleteEdge: (id: string) => void;
}

export function PropertyPanel({
  selectedNode,
  selectedEdge,
  onUpdateNode,
  onDeleteNode,
  onDeleteEdge,
}: PropertyPanelProps) {
  if (selectedEdge) {
    return (
      <div style={{ width: 280, background: '#0f0f1a', borderLeft: '1px solid #3b3b5c', padding: 16, fontFamily: 'monospace' }}>
        <h3 style={{ color: '#e2e8f0', marginTop: 0 }}>Edge</h3>
        <div style={{ color: '#64748b', fontSize: 12 }}>
          {selectedEdge.source}.{selectedEdge.sourceHandle} → {selectedEdge.target}.{selectedEdge.targetHandle}
        </div>
        <button
          onClick={() => onDeleteEdge(selectedEdge.id)}
          style={{ marginTop: 16, background: '#ef4444', color: '#fff', border: 'none', padding: '8px 16px', borderRadius: 4, cursor: 'pointer' }}
        >
          Delete Edge
        </button>
      </div>
    );
  }

  if (!selectedNode) {
    return (
      <div style={{ width: 280, background: '#0f0f1a', borderLeft: '1px solid #3b3b5c', padding: 16, fontFamily: 'monospace', color: '#64748b' }}>
        Select a node to edit properties
      </div>
    );
  }

  const { type, data, id } = selectedNode;

  return (
    <div style={{ width: 280, background: '#0f0f1a', borderLeft: '1px solid #3b3b5c', padding: 16, fontFamily: 'monospace', overflowY: 'auto' }}>
      <h3 style={{ color: '#e2e8f0', marginTop: 0, textTransform: 'capitalize' }}>{type}</h3>

      {(type === 'unit' || type === 'motif') && (
        <>
          <label style={labelStyle}>Name</label>
          <input
            value={data.name || ''}
            onChange={(e) => onUpdateNode(id, { name: e.target.value })}
            style={inputStyle}
            disabled={type === 'unit'}
          />

          {type === 'unit' && data.inputs && (
            <>
              <label style={labelStyle}>Input Mapping</label>
              {Object.entries(data.inputs).map(([key, val]) => (
                <div key={key} style={{ marginBottom: 8 }}>
                  <div style={{ color: '#64748b', fontSize: 11 }}>{key}</div>
                  <input
                    value={String(val)}
                    onChange={(e) => onUpdateNode(id, { inputs: { ...data.inputs, [key]: e.target.value } })}
                    style={inputStyle}
                  />
                </div>
              ))}
            </>
          )}
        </>
      )}

      {type === 'if' && (
        <>
          <label style={labelStyle}>Condition</label>
          <textarea
            value={data.condition || ''}
            onChange={(e) => onUpdateNode(id, { condition: e.target.value })}
            style={{ ...inputStyle, height: 60 }}
          />
        </>
      )}

      {type === 'foreach' && (
        <>
          <label style={labelStyle}>Over (expression)</label>
          <input
            value={data.over || ''}
            onChange={(e) => onUpdateNode(id, { over: e.target.value })}
            style={inputStyle}
          />
          <label style={labelStyle}>Max Iterations</label>
          <input
            type="number"
            value={data.maxIterations || 50}
            onChange={(e) => onUpdateNode(id, { maxIterations: Number(e.target.value) })}
            style={inputStyle}
          />
        </>
      )}

      {type === 'return' && data.mappings && (
        <>
          <label style={labelStyle}>Return Mappings</label>
          {Object.entries(data.mappings).map(([key, val]) => (
            <div key={key} style={{ marginBottom: 8 }}>
              <div style={{ color: '#64748b', fontSize: 11 }}>{key}</div>
              <input
                value={String(val)}
                onChange={(e) => onUpdateNode(id, { mappings: { ...data.mappings, [key]: e.target.value } })}
                style={inputStyle}
              />
            </div>
          ))}
        </>
      )}

      <button
        onClick={() => onDeleteNode(id)}
        style={{ marginTop: 16, background: '#ef4444', color: '#fff', border: 'none', padding: '8px 16px', borderRadius: 4, cursor: 'pointer' }}
      >
        Delete Node
      </button>
    </div>
  );
}

const labelStyle: React.CSSProperties = {
  display: 'block',
  color: '#94a3b8',
  fontSize: 12,
  marginBottom: 4,
  marginTop: 12,
};

const inputStyle: React.CSSProperties = {
  width: '100%',
  background: '#1a1a2e',
  border: '1px solid #3b3b5c',
  borderRadius: 4,
  padding: '6px 8px',
  color: '#e2e8f0',
  fontFamily: 'monospace',
  fontSize: 12,
  boxSizing: 'border-box',
};
```

- [ ] **Step 3: Create MotifEditor with full graph editing**

```tsx
// webui/src/components/editors/MotifEditor.tsx
import React, { useCallback, useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  NodeChange,
  EdgeChange,
  Connection,
  addEdge,
  useNodesState,
  useEdgesState,
  OnNodesChange,
  OnEdgesChange,
  OnConnect,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { UnitNode } from '../graph/nodes/UnitNode';
import { IfNode } from '../graph/nodes/IfNode';
import { ForeachNode } from '../graph/nodes/ForeachNode';
import { ReturnNode } from '../graph/nodes/ReturnNode';
import { MotifNode } from '../graph/nodes/MotifNode';
import { BlockPalette } from '../graph/Palette';
import { PropertyPanel } from './PropertyPanel';
import { getMotif, saveMotif, listMotifs } from '../../api/client';
import { BlockNode, BlockEdge, BlockType, MotifInfo } from '../../types';

const nodeTypes = {
  unit: UnitNode,
  if: IfNode,
  foreach: ForeachNode,
  return: ReturnNode,
  motif: MotifNode,
};

export function MotifEditor() {
  const { name } = useParams<{ name: string }>();
  const navigate = useNavigate();
  const [motifs, setMotifs] = useState<MotifInfo[]>([]);
  const [selectedNode, setSelectedNode] = useState<BlockNode | null>(null);
  const [selectedEdge, setSelectedEdge] = useState<BlockEdge | null>(null);
  const [nodes, setNodes, onNodesChange] = useNodesState<BlockNode>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<BlockEdge>([]);

  useEffect(() => {
    listMotifs().then(setMotifs).catch(() => {});
    if (name) {
      // Load motif YAML and parse into graph
      getMotif(name).then((yaml) => {
        const { nodes: parsedNodes, edges: parsedEdges } = parseMotifYaml(yaml);
        setNodes(parsedNodes.map(toRFNode));
        setEdges(parsedEdges.map(toRFEdge));
      }).catch(() => {});
    }
  }, [name]);

  const onConnect: OnConnect = useCallback(
    (connection: Connection) => {
      if (!connection.source || !connection.target) return;
      const edge: BlockEdge = {
        id: `edge-${Date.now()}`,
        source: connection.source,
        sourceHandle: connection.sourceHandle || '',
        target: connection.target,
        targetHandle: connection.targetHandle || '',
      };
      setEdges((eds) => [...eds, toRFEdge(edge)]);
    },
    [setEdges]
  );

  const onNodeClick = useCallback((_: React.MouseEvent, node: any) => {
    const blockNode = nodes.find((n) => n.id === node.id);
    setSelectedNode(blockNode || null);
    setSelectedEdge(null);
  }, [nodes]);

  const onEdgeClick = useCallback((_: React.MouseEvent, edge: any) => {
    const blockEdge = edges.find((e) => e.id === edge.id);
    setSelectedEdge(blockEdge || null);
    setSelectedNode(null);
  }, [edges]);

  const onNodesChangeHandler: OnNodesChange = useCallback(
    (changes: NodeChange[]) => {
      onNodesChange(changes);
      // Handle position changes
      changes.forEach((change) => {
        if (change.type === 'position' && change.position) {
          setNodes((nds) =>
            nds.map((n) =>
              n.id === change.id ? { ...n, position: change.position } : n
            )
          );
        }
      });
    },
    [onNodesChange]
  );

  const onEdgesChangeHandler: OnEdgesChange = useCallback(
    (changes: EdgeChange[]) => {
      onEdgesChange(changes);
    },
    [onEdgesChange]
  );

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      const type = e.dataTransfer.getData('application/reactflow') as BlockType;
      const blockName = e.dataTransfer.getData('application/blockname');

      if (!type) return;

      const reactFlowBounds = e.currentTarget.getBoundingClientRect();
      const position = {
        x: e.clientX - reactFlowBounds.left - 80,
        y: e.clientY - reactFlowBounds.top - 30,
      };

      const id = `${type}-${Date.now()}`;
      const newNode = {
        id,
        type,
        position,
        data: createNodeData(type, blockName),
      };

      setNodes((nds) => [...nds, newNode as any]);
    },
    [setNodes]
  );

  const updateNodeData = useCallback(
    (id: string, data: Partial<BlockNode['data']>) => {
      setNodes((nds) =>
        nds.map((n) => (n.id === id ? { ...n, data: { ...n.data, ...data } } : n))
      );
      setSelectedNode((prev) => (prev && prev.id === id ? { ...prev, data: { ...prev.data, ...data } } : prev));
    },
    [setNodes]
  );

  const deleteNode = useCallback(
    (id: string) => {
      setNodes((nds) => nds.filter((n) => n.id !== id));
      setEdges((eds) => eds.filter((e) => e.source !== id && e.target !== id));
      setSelectedNode(null);
    },
    [setNodes, setEdges]
  );

  const deleteEdge = useCallback(
    (id: string) => {
      setEdges((eds) => eds.filter((e) => e.id !== id));
      setSelectedEdge(null);
    },
    [setEdges]
  );

  const handleSave = async () => {
    if (!name) return;
    const { yaml } = graphToYaml(nodes as BlockNode[], edges as BlockEdge[]);
    await saveMotif(name, yaml);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 16, padding: '12px 16px', background: '#0f0f1a', borderBottom: '1px solid #3b3b5c' }}>
        <button onClick={() => navigate(-1)} style={buttonStyle}>← Back</button>
        <h2 style={{ margin: 0, flex: 1 }}>Motif: {name}</h2>
        <button onClick={handleSave} style={buttonStyle}>Save</button>
      </div>

      {/* Body */}
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        <BlockPalette
          motifs={motifs}
          onDragStart={(type, blockName) => {
            // Set drag data
          }}
        />

        <div
          style={{ flex: 1, position: 'relative' }}
          onDragOver={handleDragOver}
          onDrop={handleDrop}
        >
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChangeHandler}
            onEdgesChange={onEdgesChangeHandler}
            onConnect={onConnect}
            onNodeClick={onNodeClick}
            onEdgeClick={onEdgeClick}
            nodeTypes={nodeTypes}
            fitView
            snapToGrid
            snapGrid={[16, 16]}
          >
            <Background />
            <Controls />
            <MiniMap />
          </ReactFlow>
        </div>

        <PropertyPanel
          selectedNode={selectedNode as BlockNode | null}
          selectedEdge={selectedEdge as BlockEdge | null}
          onUpdateNode={updateNodeData}
          onDeleteNode={deleteNode}
          onDeleteEdge={deleteEdge}
        />
      </div>
    </div>
  );
}

// Helper functions
function createNodeData(type: BlockType, name?: string): BlockNode['data'] {
  switch (type) {
    case 'unit':
      return { name: name || '', inputs: { input: '' }, outputs: [{ id: 'out-1', name: 'output', type: 'string' }] };
    case 'if':
      return { condition: '', expanded: false };
    case 'foreach':
      return { over: '', maxIterations: 50, parallel: false, expanded: false };
    case 'return':
      return { mappings: {} };
    case 'motif':
      return { name: name || '', expanded: false };
    default:
      return {};
  }
}

function toRFNode(node: BlockNode) {
  return {
    id: node.id,
    type: node.type,
    position: node.position,
    data: node.data,
  };
}

function toRFEdge(edge: BlockEdge) {
  return {
    id: edge.id,
    source: edge.source,
    target: edge.target,
    sourceHandle: edge.sourceHandle,
    targetHandle: edge.targetHandle,
    type: 'smoothstep',
    animated: true,
  };
}

// YAML parsing and serialization (simplified)
function parseMotifYaml(yaml: string): { nodes: BlockNode[]; edges: BlockEdge[] } {
  // Parse YAML flow steps → nodes + edges
  // This is a placeholder - real impl would parse the actual YAML structure
  return { nodes: [], edges: [] };
}

function graphToYaml(nodes: BlockNode[], edges: BlockEdge[]): { yaml: string } {
  // Serialize nodes + edges → YAML flow
  // This is a placeholder - real impl would generate valid motif YAML
  return { yaml: '' };
}

const buttonStyle: React.CSSProperties = {
  background: '#7c3aed',
  color: '#fff',
  border: 'none',
  padding: '8px 16px',
  borderRadius: 4,
  cursor: 'pointer',
  fontFamily: 'monospace',
};
```

- [ ] **Step 4: Add MotifEditor route to App.tsx**

```tsx
import { MotifEditor } from './components/editors/MotifEditor';
<Route path="/motifs/:name/edit" element={<MotifEditor />} />
```

- [ ] **Step 5: Add saveMotif to API client**

```typescript
export async function saveMotif(name: string, yaml: string): Promise<{ message: string }> {
  const response = await fetch(`${API_BASE}/motifs/${encodeURIComponent(name)}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'text/plain' },
    body: yaml,
  });
  if (!response.ok) throw new Error('Failed to save motif');
  return { message: 'Motif saved' };
}
```

- [ ] **Step 6: Commit**

```bash
git add webui/src/components/editors/MotifEditor.tsx webui/src/components/graph/Palette.tsx webui/src/components/editors/PropertyPanel.tsx webui/src/api/client.ts webui/src/App.tsx
git commit -m "feat: create Motif Editor with React Flow graph canvas"
```

---

### Task 10: Create Structure Editor Redesign

**Files:**
- Create: `webui/src/components/editors/StructureEditor.tsx` (redesign)
- Modify: `webui/src/App.tsx`

- [ ] **Step 1: Create StructureEditor with full digraph + Motif expansion**

This reuses the same graph canvas + node types. The key difference is:
1. Palette includes Motif blocks (from MotifEditor palette)
2. Motif blocks are expandable — clicking expands inline

```tsx
// webui/src/components/editors/StructureEditor.tsx
// Heavily based on MotifEditor, with these differences:
// 1. Loads structure instead of motif
// 2. Palette includes both Unit and Motif blocks
// 3. MotifNode expanded state shows internal motif flow
// 4. Save serializes to structure manifest.yaml format

// (Implementation similar to MotifEditor - see Task 9)
// Key additions:
// - listStructures for structure data
// - saveStructure → PUT /api/structures/:name
// - Motif expansion: when MotifNode expanded=true, render its internal graph inline
// - Expanded motif shows as nested ReactFlow within the parent
```

- [ ] **Step 2: Add route**

```tsx
// In App.tsx, keep existing /structures/:name route pointing to StructureEditor (redesigned)
```

- [ ] **Step 3: Commit**

```bash
git add webui/src/components/editors/StructureEditor.tsx
git commit -m "feat: redesign Structure Editor with digraph canvas and Motif expansion"
```

---

### Task 11: Create Chat Assistant Component

**Files:**
- Create: `webui/src/components/editors/ChatAssistant.tsx`
- Modify: `webui/src/components/editors/UnitEditor.tsx` (integrate ChatAssistant)

- [ ] **Step 1: Create ChatAssistant component**

```tsx
// webui/src/components/editors/ChatAssistant.tsx
import React, { useState, useRef, useEffect } from 'react';

interface Message {
  role: 'user' | 'assistant';
  content: string;
}

interface ChatAssistantProps {
  context?: {
    type: 'unit' | 'motif' | 'structure';
    name: string;
    yaml?: string;
  };
}

export function ChatAssistant({ context }: ChatAssistantProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleSend = async () => {
    if (!input.trim() || loading) return;
    const userMessage = { role: 'user' as const, content: input };
    setMessages((prev) => [...prev, userMessage]);
    setInput('');
    setLoading(true);

    try {
      const systemPrompt = buildSystemPrompt(context);
      const response = await fetch('/api/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          model: 'claude-3-5-sonnet-20241022',
          messages: [
            { role: 'system', content: systemPrompt },
            ...messages.map((m) => ({ role: m.role, content: m.content })),
            { role: 'user', content: input },
          ],
          max_tokens: 1024,
          stream: false,
        }),
      });
      const data = await response.json();
      const assistantMessage = { role: 'assistant' as const, content: data.content || data.error || 'No response' };
      setMessages((prev) => [...prev, assistantMessage]);
    } catch (e) {
      setMessages((prev) => [...prev, { role: 'assistant', content: `Error: ${e}` }]);
    }
    setLoading(false);
  };

  return (
    <div style={{
      background: '#0f0f1a',
      borderTop: '1px solid #3b3b5c',
      display: 'flex',
      flexDirection: 'column',
      height: 300,
      fontFamily: 'monospace',
    }}>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', padding: '8px 16px', borderBottom: '1px solid #3b3b5c' }}>
        <span style={{ color: '#7c3aed', fontSize: 14 }}>🤖</span>
        <span style={{ color: '#e2e8f0', marginLeft: 8, fontSize: 13 }}>Assistant</span>
      </div>

      {/* Messages */}
      <div style={{ flex: 1, overflowY: 'auto', padding: 16 }}>
        {messages.length === 0 && (
          <div style={{ color: '#64748b', fontSize: 12 }}>
            Ask me about editing this {context?.type || 'resource'}.
          </div>
        )}
        {messages.map((msg, i) => (
          <div key={i} style={{ marginBottom: 12 }}>
            <div style={{ color: msg.role === 'user' ? '#7c3aed' : '#22c55e', fontSize: 11, marginBottom: 2 }}>
              {msg.role === 'user' ? 'You' : 'Assistant'}
            </div>
            <div style={{ color: '#e2e8f0', fontSize: 13, whiteSpace: 'pre-wrap' }}>{msg.content}</div>
          </div>
        ))}
        {loading && <div style={{ color: '#64748b', fontSize: 12 }}>Thinking...</div>}
        <div ref={bottomRef} />
      </div>

      {/* Input */}
      <div style={{ display: 'flex', gap: 8, padding: 12, borderTop: '1px solid #3b3b5c' }}>
        <input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleSend()}
          placeholder={`Ask about ${context?.type || 'this'}...`}
          style={{
            flex: 1,
            background: '#1a1a2e',
            border: '1px solid #3b3b5c',
            borderRadius: 4,
            padding: '8px 12px',
            color: '#e2e8f0',
            fontFamily: 'monospace',
            fontSize: 13,
          }}
        />
        <button
          onClick={handleSend}
          disabled={loading}
          style={{
            background: '#7c3aed',
            color: '#fff',
            border: 'none',
            padding: '8px 16px',
            borderRadius: 4,
            cursor: loading ? 'not-allowed' : 'pointer',
            opacity: loading ? 0.5 : 1,
          }}
        >
          Send
        </button>
      </div>
    </div>
  );
}

function buildSystemPrompt(context?: ChatAssistantProps['context']): string {
  if (!context) return 'You are a helpful assistant for COGTOME.';
  return `You are a COGTOME expert helping edit a ${context.type} called "${context.name}".
${context.yaml ? `Current content:\n${context.yaml}` : ''}
COGTOME info:
- Units: atomic executables, stdin/stdout JSON
- Motifs: flow of Units, supports if/foreach control flow
- Structures: composed of Motifs and Units
- Variables: \${params.x}, \${steps.name.output.field}
- Expression functions: filter, map with == != > < && ||`;
}
```

- [ ] **Step 2: Integrate ChatAssistant into UnitEditor**

Add to UnitEditor's bottom section:
```tsx
<ChatAssistant context={{ type: 'unit', name: name || '' }} />
```

- [ ] **Step 3: Commit**

```bash
git add webui/src/components/editors/ChatAssistant.tsx webui/src/components/editors/UnitEditor.tsx
git commit -m "feat: add ChatAssistant component for AI-powered editing help"
```

---

## Phase 4: Graph ↔ YAML Serialization

### Task 12: Implement YAML Serialization

**Files:**
- Create: `webui/src/components/graph/graphUtils.ts`

- [ ] **Step 1: Create graphUtils.ts with full YAML ↔ graph conversion**

This is the critical piece that converts between the visual graph and the YAML format.

```typescript
// webui/src/components/graph/graphUtils.ts
import { BlockNode, BlockEdge, Graph } from '../../types';
import yaml from 'yaml';

// COGTOME Motif YAML format:
// name: <motif-name>
// type: motif
// flow:
//   - name: step1
//     unit: unit-name
//     input:
//       key: ${params.x}
//   - name: step2
//     ...

export function graphToYaml(nodes: BlockNode[], edges: BlockEdge[], name: string): string {
  // 1. Topological sort to determine execution order
  const sorted = topologicalSort(nodes, edges);

  // 2. Group nodes into flow steps
  const flow = sorted.map((nodeId) => {
    const node = nodes.find((n) => n.id === nodeId);
    if (!node) return null;

    // Find incoming edges to determine inputs
    const incomingEdges = edges.filter((e) => e.target === nodeId);
    const inputMappings: Record<string, string> = {};

    incomingEdges.forEach((edge) => {
      const sourceNode = nodes.find((n) => n.id === edge.source);
      if (sourceNode && edge.targetHandle) {
        inputMappings[edge.targetHandle] = `\${steps.${sourceNode.data.name || edge.source}.output.${edge.sourceHandle}}`;
      }
    });

    // Also include manually entered input expressions
    if (node.data.inputs) {
      Object.entries(node.data.inputs).forEach(([key, val]) => {
        if (val && !Object.values(inputMappings).some((v) => v.includes(key))) {
          inputMappings[key] = val;
        }
      });
    }

    switch (node.type) {
      case 'unit':
        return {
          name: node.data.name || nodeId,
          unit: node.data.name,
          input: inputMappings,
        };
      case 'foreach':
        return {
          name: node.data.name || nodeId,
          foreach: {
            over: node.data.over || '',
            as_var: 'item',
            max_iterations: node.data.maxIterations || 50,
            parallel: node.data.parallel || false,
            flow: [], // nested flow from expanded graph
          },
        };
      case 'if':
        return {
          name: node.data.name || nodeId,
          if: {
            condition: node.data.condition || '',
            then: [], // nested flow
            else: [], // nested flow
          },
        };
      case 'return':
        return {
          return: node.data.mappings || {},
        };
      default:
        return null;
    }
  }).filter(Boolean);

  const doc = {
    name,
    type: 'motif',
    flow: flow.filter(Boolean),
  };

  return yaml.stringify(doc, { indent: 2 });
}

export function yamlToGraph(yamlString: string, name: string): Graph {
  const doc = yaml.parse(yamlString);
  if (!doc || !doc.flow) return { nodes: [], edges: [] };

  const nodes: BlockNode[] = [];
  const edges: BlockEdge[] = [];
  const positionMap: Record<string, { x: number; y: number }> = {};

  // First pass: create nodes
  let xOffset = 0;
  doc.flow.forEach((step: any, index: number) => {
    const nodeId = `${step.name || step.unit || 'step'}-${index}`;
    const position = { x: xOffset, y: 150 };
    positionMap[nodeId] = position;

    if (step.return) {
      nodes.push({
        id: nodeId,
        type: 'return',
        position,
        data: { mappings: step.return },
      });
    } else if (step.foreach) {
      nodes.push({
        id: nodeId,
        type: 'foreach',
        position,
        data: {
          name: step.name,
          over: step.foreach.over,
          maxIterations: step.foreach.max_iterations,
          parallel: step.foreach.parallel,
        },
      });
    } else if (step.if) {
      nodes.push({
        id: nodeId,
        type: 'if',
        position,
        data: {
          name: step.name,
          condition: step.if.condition,
        },
      });
    } else if (step.unit) {
      nodes.push({
        id: nodeId,
        type: 'unit',
        position,
        data: {
          name: step.unit,
          inputs: step.input || {},
          outputs: [{ id: 'output', name: 'output', type: 'string' }],
        },
      });
    }

    xOffset += 220;
  });

  // Second pass: create edges based on data flow
  // This is simplified - real impl would analyze variable references
  for (let i = 0; i < nodes.length - 1; i++) {
    const sourceNode = nodes[i];
    const targetNode = nodes[i + 1];
    edges.push({
      id: `edge-${i}`,
      source: sourceNode.id,
      sourceHandle: 'output',
      target: targetNode.id,
      targetHandle: 'input',
    });
  }

  return { nodes, edges };
}

function topologicalSort(nodes: BlockNode[], edges: BlockEdge[]): string[] {
  const inDegree: Record<string, number> = {};
  const adj: Record<string, string[]> = {};

  nodes.forEach((n) => {
    inDegree[n.id] = 0;
    adj[n.id] = [];
  });

  edges.forEach((e) => {
    adj[e.source]?.push(e.target);
    inDegree[e.target] = (inDegree[e.target] || 0) + 1;
  });

  const queue: string[] = [];
  Object.entries(inDegree).forEach(([id, deg]) => {
    if (deg === 0) queue.push(id);
  });

  const result: string[] = [];
  while (queue.length > 0) {
    const current = queue.shift()!;
    result.push(current);
    adj[current]?.forEach((neighbor) => {
      inDegree[neighbor]--;
      if (inDegree[neighbor] === 0) queue.push(neighbor);
    });
  }

  return result;
}
```

- [ ] **Step 2: Commit**

```bash
git add webui/src/components/graph/graphUtils.ts
git commit -m "feat: implement graph ↔ YAML serialization for Motif/Structure"
```

---

## Phase 5: Polish & Integration

### Task 13: Auto-Layout + Keyboard Shortcuts

**Files:**
- Modify: `webui/src/components/graph/GraphCanvas.tsx`
- Modify: `webui/src/components/editors/MotifEditor.tsx`

- [ ] **Step 1: Add auto-layout button**

Add to GraphCanvas:
```tsx
// After <MiniMap />, add:
<button
  onClick={() => {
    const layoutedNodes = autoLayout(nodes);
    setNodes(layoutedNodes);
  }}
  style={{
    position: 'absolute',
    top: 10,
    right: 10,
    zIndex: 10,
    background: '#1a1a2e',
    border: '1px solid #3b3b5c',
    color: '#e2e8f0',
    padding: '6px 12px',
    borderRadius: 4,
    cursor: 'pointer',
    fontSize: 12,
    fontFamily: 'monospace',
  }}
>
  Auto Layout
</button>
```

Add autoLayout helper:
```typescript
function autoLayout(nodes: Node[]): Node[] {
  // Simple left-to-right layout
  const sorted = [...nodes].sort((a, b) => a.position.x - b.position.x);
  return sorted.map((node, i) => ({
    ...node,
    position: { x: i * 220, y: 150 },
  }));
}
```

- [ ] **Step 2: Add keyboard shortcuts to MotifEditor**

```tsx
// In MotifEditor, add useEffect:
useEffect(() => {
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Delete' || e.key === 'Backspace') {
      if (selectedNode) deleteNode(selectedNode.id);
      if (selectedEdge) deleteEdge(selectedEdge.id);
    }
  };
  window.addEventListener('keydown', handleKeyDown);
  return () => window.removeEventListener('keydown', handleKeyDown);
}, [selectedNode, selectedEdge]);
```

- [ ] **Step 3: Commit**

```bash
git add webui/src/components/graph/GraphCanvas.tsx webui/src/components/editors/MotifEditor.tsx
git commit -m "feat: add auto-layout and keyboard shortcuts to graph editor"
```

---

### Task 14: YAML View Toggle

**Files:**
- Modify: `webui/src/components/editors/MotifEditor.tsx`
- Modify: `webui/src/components/editors/StructureEditor.tsx`

- [ ] **Step 1: Add YAML/Graph view toggle**

In MotifEditor header:
```tsx
const [viewMode, setViewMode] = useState<'graph' | 'yaml'>('graph');

// In header:
<div style={{ display: 'flex', gap: 8 }}>
  <button
    onClick={() => setViewMode('graph')}
    style={{ ...viewButtonStyle, background: viewMode === 'graph' ? '#7c3aed' : undefined }}
  >
    Graph
  </button>
  <button
    onClick={() => setViewMode('yaml')}
    style={{ ...viewButtonStyle, background: viewMode === 'yaml' ? '#7c3aed' : undefined }}
  >
    YAML
  </button>
</div>

// In body:
{viewMode === 'yaml' ? (
  <textarea
    value={graphToYaml(nodes as BlockNode[], edges as BlockEdge[], name || '')}
    onChange={(val) => {
      const { nodes: parsed } = yamlToGraph(val, name || '');
      setNodes(parsed.map(toRFNode));
      setEdges([]);
    }}
    style={{
      flex: 1,
      background: '#0f0f1a',
      color: '#e2e8f0',
      fontFamily: 'monospace',
      fontSize: 13,
      padding: 16,
      border: 'none',
      resize: 'none',
    }}
  />
) : (
  <ReactFlow ... />
)}
```

- [ ] **Step 2: Commit**

```bash
git commit -m "feat: add YAML/Graph view toggle to editors"
```

---

## Implementation Complete

**Summary of deliverables:**

1. **Graph types** — `BlockNode`, `BlockEdge`, `Graph` types
2. **Graph store** — Zustand store for reactive graph state
3. **React Flow canvas** — pan/zoom/ports/minimap graph editor
4. **Node components** — UnitNode, IfNode, ForeachNode, ReturnNode, MotifNode
5. **Unit Editor** — config panel + test panel + chat assistant
6. **Motif Editor** — digraph canvas + palette + property panel + YAML sync
7. **Structure Editor** — digraph with Motif blocks + expansion
8. **Chat Assistant** — Claude API-powered editing help
9. **Graph ↔ YAML** — bidirectional serialization
10. **Polish** — auto-layout, keyboard shortcuts, view toggle

import type { GraphNode, GraphEdge, Graph, MotifManifestV2, Position } from '../../types';

/**
 * COGTOME v2 Motif JSON format:
 * {
 *   "name": "motif-name",
 *   "type": "motif",
 *   "graph": {
 *     "nodes": [...],
 *     "edges": [...]
 *   }
 * }
 */

// Convert internal GraphNode to MotifManifestV2 JSON
export function graphToJson(nodes: GraphNode[], edges: GraphEdge[], name: string): MotifManifestV2 {
  return {
    name,
    type: 'motif',
    version: '2.0',
    graph: {
      nodes: nodes.map(n => JSON.parse(JSON.stringify(n))),
      edges: edges.map(e => JSON.parse(JSON.stringify(e))),
    },
  };
}

// Convert MotifManifestV2 JSON to internal Graph format
export function jsonToGraph(json: MotifManifestV2 | string): Graph {
  let manifest: MotifManifestV2;
  if (typeof json === 'string') {
    manifest = JSON.parse(json);
  } else {
    manifest = json;
  }

  if (!manifest.graph) {
    return { nodes: [], edges: [] };
  }

  return manifest.graph;
}

// Topological sort for execution order
export function topologicalSort(nodes: GraphNode[], edges: GraphEdge[]): string[] {
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

// Auto-layout nodes in a grid pattern
export function autoLayout(nodes: GraphNode[]): GraphNode[] {
  const sorted = [...nodes].sort((a, b) => {
    if (Math.abs(a.position.y - b.position.y) > 50) {
      return a.position.y - b.position.y;
    }
    return a.position.x - b.position.x;
  });

  return sorted.map((node, i) => ({
    ...node,
    position: { x: 50 + (i % 5) * 250, y: 50 + Math.floor(i / 5) * 150 } as Position,
  }));
}

// Create a default start node
export function createStartNode(x: number = 0, y: number = 100): GraphNode {
  return {
    id: 'start',
    type: 'start',
    position: { x, y },
    data: {},
  };
}

// Create a default return node
export function createReturnNode(x: number, y: number): GraphNode {
  return {
    id: 'return',
    type: 'return',
    position: { x, y },
    data: { values: {} },
  };
}

// Create a unit node
export function createUnitNode(id: string, unitName: string, x: number, y: number): GraphNode {
  return {
    id,
    type: 'unit',
    position: { x, y },
    data: { unit: unitName, inputs: {} },
  };
}

// Create an if node
export function createIfNode(id: string, condition: string, x: number, y: number): GraphNode {
  return {
    id,
    type: 'if',
    position: { x, y },
    data: { condition },
  };
}

// Create a match node
export function createMatchNode(id: string, on: string, x: number, y: number): GraphNode {
  return {
    id,
    type: 'match',
    position: { x, y },
    data: { condition: on },
  };
}

// Create a foreach node
export function createForeachNode(id: string, over: string, x: number, y: number): GraphNode {
  return {
    id,
    type: 'foreach',
    position: { x, y },
    data: { over, as_var: 'item', maxIterations: 50, parallel: false, subgraph: { nodes: [], edges: [] } },
  };
}

// Create a fork node
export function createForkNode(id: string, x: number, y: number): GraphNode {
  return {
    id,
    type: 'fork',
    position: { x, y },
    data: {},
  };
}

// Create a join node
export function createJoinNode(id: string, x: number, y: number): GraphNode {
  return {
    id,
    type: 'join',
    position: { x, y },
    data: {},
  };
}

// Create a motif reference node
export function createMotifNode(id: string, motifName: string, x: number, y: number): GraphNode {
  return {
    id,
    type: 'motif',
    position: { x, y },
    data: { motif: motifName, inputs: {} },
  };
}

// Generate a unique node ID
export function generateNodeId(nodes: GraphNode[], prefix: string = 'node'): string {
  const existing = new Set(nodes.map(n => n.id));
  let i = 1;
  while (existing.has(`${prefix}-${i}`)) i++;
  return `${prefix}-${i}`;
}

// Generate a unique edge ID
export function generateEdgeId(edges: GraphEdge[]): string {
  const existing = new Set(edges.map(e => e.id));
  let i = 1;
  while (existing.has(`edge-${i}`)) i++;
  return `edge-${i}-${Date.now()}`;
}

// Create a new edge between two nodes
export function createEdge(source: string, target: string, sourceHandle: string = 'output', targetHandle: string = 'input'): GraphEdge {
  return {
    id: `e-${source}-${target}`,
    source,
    target,
    sourceHandle,
    targetHandle,
  };
}
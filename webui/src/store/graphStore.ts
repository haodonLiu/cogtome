import { create } from 'zustand';
import type { GraphNode, GraphEdge, Graph, BlockType, Position } from '../types';

interface GraphStore {
  // Graph data
  nodes: GraphNode[];
  edges: GraphEdge[];
  selectedNodeId: string | null;
  selectedEdgeId: string | null;

  // Actions
  setGraph: (graph: Graph) => void;
  addNode: (type: BlockType, position: Position, name?: string, data?: Partial<GraphNode['data']>) => void;
  updateNode: (id: string, data: Partial<GraphNode['data']>) => void;
  removeNode: (id: string) => void;
  addEdge: (source: string, sourceHandle: string, target: string, targetHandle: string, label?: string) => void;
  removeEdge: (id: string) => void;
  selectNode: (id: string | null) => void;
  selectEdge: (id: string | null) => void;
  updateNodePosition: (id: string, position: Position) => void;
  clearGraph: () => void;

  // Serialization
  toGraph: () => Graph;
}

function createDefaultNodeData(type: BlockType): GraphNode['data'] {
  switch (type) {
    case 'unit':
      return { unit: '', inputs: {} };
    case 'if':
      return { condition: '' };
    case 'match':
      return { condition: '' };
    case 'foreach':
      return { over: '', as_var: 'item', maxIterations: 50, parallel: false, subgraph: { nodes: [], edges: [] } };
    case 'fork':
      return {};
    case 'join':
      return {};
    case 'return':
      return { values: {} };
    case 'motif':
      return { motif: '', inputs: {} };
    case 'start':
      return {};
    default:
      return {};
  }
}

export const useGraphStore = create<GraphStore>((set, get) => ({
  nodes: [],
  edges: [],
  selectedNodeId: null,
  selectedEdgeId: null,

  setGraph: (graph) => set({ nodes: graph.nodes, edges: graph.edges }),

  addNode: (type, position, name = '', data = {}) => {
    const id = `${type}-${Date.now()}`;
    const defaultData = createDefaultNodeData(type);
    const newNode: GraphNode = {
      id,
      type,
      position,
      data: { ...defaultData, name, ...data },
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

  addEdge: (source, sourceHandle, target, targetHandle, label) => {
    const id = `edge-${source}-${target}`;
    const newEdge: GraphEdge = { id, source, sourceHandle, target, targetHandle, label };
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
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

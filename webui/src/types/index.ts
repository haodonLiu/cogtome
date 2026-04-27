// ============================================================================
// COGTOME v2.0 Types
// ============================================================================

export interface MotifRef {
  name: string;
}

export interface StructureManifest {
  name: string;
  type?: string;
  motifs: MotifRef[];
  input_schema?: JsonSchema;
  output_schema?: JsonSchema;
}

export interface JsonSchema {
  type?: string;
  properties?: Record<string, JsonSchema>;
  required?: string[];
  [key: string]: unknown;
}

export interface StructureInfo {
  name: string;
  path: string;
  motif_count: number;
}

export interface MotifInfo {
  name: string;
  path: string;
  step_count: number;
}

export interface UnitInfo {
  name: string;
  path: string;
  timeout?: number;
  concurrency?: number;
  description?: string;
}

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
}

export interface ValidationError {
  path: string;
  message: string;
}

// ============================================================================
// Graph Types for Visual Editor (v2)
// ============================================================================

// Node types matching COGTOME v2 spec
export type NodeType = 'start' | 'unit' | 'if' | 'match' | 'foreach' | 'fork' | 'join' | 'return' | 'motif';

export interface Position {
  x: number;
  y: number;
}

export interface Port {
  id: string;
  name: string;
  type: 'string' | 'number' | 'boolean' | 'array' | 'object';
}

// Graph node for React Flow / visual editor
export interface GraphNode {
  id: string;
  type: NodeType;
  position: Position;
  data: {
    name?: string;
    unit?: string;
    motif?: string;
    inputs?: Record<string, string>;
    outputs?: Port[];
    condition?: string;
    over?: string;
    maxIterations?: number;
    parallel?: boolean;
    mappings?: Record<string, string>;
    values?: Record<string, string>;
    expanded?: boolean;
    internalGraph?: Graph;
    subgraph?: Graph;
    as_var?: string;
    [key: string]: unknown;
  };
}

// Graph edge for React Flow
export interface GraphEdge {
  id: string;
  source: string;
  sourceHandle: string;
  target: string;
  targetHandle: string;
  label?: string;
}

// Graph container
export interface Graph {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

// Full Motif manifest v2
export interface MotifManifestV2 {
  name: string;
  type: 'motif';
  version?: string;
  description?: string;
  required_units?: string[];
  graph: Graph;
  input_schema?: JsonSchema;
  output_schema?: JsonSchema;
}

// Backward compatibility aliases
export type BlockType = NodeType;
export type BlockNode = GraphNode;
export type BlockEdge = GraphEdge;
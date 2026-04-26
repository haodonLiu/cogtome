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
}

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
}

export interface ValidationError {
  path: string;
  message: string;
}

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
    name?: string;
    inputs?: Record<string, string>;
    outputs?: Port[];
    condition?: string;
    over?: string;
    maxIterations?: number;
    parallel?: boolean;
    mappings?: Record<string, string>;
    expanded?: boolean;
    internalGraph?: Graph;
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

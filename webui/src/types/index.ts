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

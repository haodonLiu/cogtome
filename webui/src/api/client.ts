import type { StructureInfo, MotifInfo, UnitInfo, StructureManifest, ValidationResult } from '../types';

const API_BASE = '/api';

async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
  const response = await fetch(url, options);
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: { message: response.statusText } }));
    throw new Error(error.error?.message || `HTTP ${response.status}`);
  }
  return response.json();
}

// Structures API
export async function listStructures(): Promise<StructureInfo[]> {
  return fetchJson<StructureInfo[]>(`${API_BASE}/structures`);
}

export async function getStructure(name: string): Promise<StructureManifest> {
  return fetchJson<StructureManifest>(`${API_BASE}/structures/${encodeURIComponent(name)}`);
}

export async function saveStructure(name: string, manifest: StructureManifest): Promise<{ message: string; path: string }> {
  return fetchJson<{ message: string; path: string }>(`${API_BASE}/structures/${encodeURIComponent(name)}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(manifest),
  });
}

export async function deleteStructure(name: string): Promise<{ message: string; name: string }> {
  return fetchJson<{ message: string; name: string }>(`${API_BASE}/structures/${encodeURIComponent(name)}`, {
    method: 'DELETE',
  });
}

// Motifs API (read-only)
export async function listMotifs(): Promise<MotifInfo[]> {
  return fetchJson<MotifInfo[]>(`${API_BASE}/motifs`);
}

export async function getMotif(name: string): Promise<string> {
  const response = await fetch(`${API_BASE}/motifs/${encodeURIComponent(name)}`);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  return response.text();
}

export async function saveMotif(name: string, yaml: string): Promise<{ message: string }> {
  const response = await fetch(`${API_BASE}/motifs/${encodeURIComponent(name)}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'text/plain' },
    body: yaml,
  });
  if (!response.ok) throw new Error('Failed to save motif');
  return { message: 'Motif saved' };
}

// Units API
export async function listUnits(): Promise<UnitInfo[]> {
  return fetchJson<UnitInfo[]>(`${API_BASE}/units`);
}

export async function getUnit(name: string): Promise<any> {
  const response = await fetch(`${API_BASE}/units/${encodeURIComponent(name)}`);
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return response.json();
}

export async function saveUnit(name: string, config: { timeout?: number; concurrency?: number; description?: string }): Promise<{ message: string }> {
  return fetchJson<{ message: string }>(`${API_BASE}/units/${encodeURIComponent(name)}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(config),
  });
}

// Validation API
export async function validateStructure(name: string): Promise<ValidationResult> {
  return fetchJson<ValidationResult>(`${API_BASE}/validate/structure/${encodeURIComponent(name)}`, {
    method: 'POST',
  });
}

export async function validateMotif(name: string): Promise<ValidationResult> {
  return fetchJson<ValidationResult>(`${API_BASE}/validate/motif/${encodeURIComponent(name)}`, {
    method: 'POST',
  });
}

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import {
  listMotifs,
  getMotif,
  saveMotif,
  listStructures,
  getStructure,
  saveStructure,
} from '../client'
import type { MotifManifestV2, StructureManifest } from '../../types'

// Mock fetch globally
const mockFetch = vi.fn()
globalThis.fetch = mockFetch as typeof fetch

describe('API Client', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('listMotifs', () => {
    it('fetches and returns list of motifs', async () => {
      const mockMotifs = [
        { name: 'test-motif-1', path: '/skills/motifs/test-motif-1.yaml', step_count: 3 },
        { name: 'test-motif-2', path: '/skills/motifs/test-motif-2.yaml', step_count: 5 },
      ]

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockMotifs),
      })

      const result = await listMotifs()

      expect(mockFetch).toHaveBeenCalledWith('/api/motifs', undefined)
      expect(result).toEqual(mockMotifs)
    })

    it('throws error on non-OK response', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        json: () => Promise.resolve({ error: { message: 'Not found' } }),
      })

      await expect(listMotifs()).rejects.toThrow('Not found')
    })
  })

  describe('getMotif', () => {
    it('fetches and parses MotifManifestV2 JSON', async () => {
      const mockManifest: MotifManifestV2 = {
        name: 'test-motif',
        type: 'motif',
        version: '2.0',
        graph: {
          nodes: [
            { id: 'start', type: 'start', position: { x: 0, y: 0 }, data: {} },
            { id: 'unit-1', type: 'unit', position: { x: 100, y: 0 }, data: { unit: 'echo' } },
          ],
          edges: [{ id: 'e1', source: 'start', target: 'unit-1', sourceHandle: 'default', targetHandle: 'default' }],
        },
      }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        text: () => Promise.resolve(JSON.stringify(mockManifest)),
      })

      const result = await getMotif('test-motif')

      expect(mockFetch).toHaveBeenCalledWith('/api/motifs/test-motif')
      expect(result.name).toBe('test-motif')
      expect(result.type).toBe('motif')
      expect(result.graph.nodes).toHaveLength(2)
      expect(result.graph.edges).toHaveLength(1)
    })

    it('returns empty graph on invalid JSON (legacy fallback)', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        text: () => Promise.resolve('invalid json'),
      })

      const result = await getMotif('legacy-motif')

      expect(result).toEqual({ name: 'legacy-motif', type: 'motif', graph: { nodes: [], edges: [] } })
    })

    it('throws error on HTTP error', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
      })

      await expect(getMotif('error-motif')).rejects.toThrow('HTTP 500')
    })
  })

  describe('saveMotif', () => {
    it('sends PUT request with JSON body', async () => {
      const manifest: MotifManifestV2 = {
        name: 'new-motif',
        type: 'motif',
        version: '2.0',
        graph: {
          nodes: [{ id: 'start', type: 'start', position: { x: 0, y: 0 }, data: {} }],
          edges: [],
        },
      }

      const expectedResponse = { message: 'Saved', path: '/skills/motifs/new-motif.yaml' }
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(expectedResponse),
      })

      const result = await saveMotif('new-motif', manifest)

      expect(mockFetch).toHaveBeenCalledWith('/api/motifs/new-motif', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(manifest),
      })
      expect(result).toEqual(expectedResponse)
    })

    it('includes correct Content-Type header', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({ message: 'OK' }),
      })

      await saveMotif('test', { name: 'test', type: 'motif', graph: { nodes: [], edges: [] } })

      const call = mockFetch.mock.calls[0]
      expect(call[1].headers['Content-Type']).toBe('application/json')
    })
  })

  describe('listStructures', () => {
    it('fetches and returns list of structures', async () => {
      const mockStructures = [
        { name: 'test-structure', path: '/skills/structures/test-structure/manifest.yaml', motif_count: 2 },
      ]

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockStructures),
      })

      const result = await listStructures()

      expect(mockFetch).toHaveBeenCalledWith('/api/structures', undefined)
      expect(result).toEqual(mockStructures)
    })
  })

  describe('getStructure', () => {
    it('fetches and returns structure manifest', async () => {
      const mockManifest: StructureManifest = {
        name: 'test-structure',
        motifs: [{ name: 'motif-1' }, { name: 'motif-2' }],
      }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockManifest),
      })

      const result = await getStructure('test-structure')

      expect(mockFetch).toHaveBeenCalledWith('/api/structures/test-structure', undefined)
      expect(result.name).toBe('test-structure')
      expect(result.motifs).toHaveLength(2)
    })
  })

  describe('saveStructure', () => {
    it('sends PUT request with structure manifest', async () => {
      const manifest: StructureManifest = {
        name: 'new-structure',
        motifs: [{ name: 'motif-1' }],
      }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({ message: 'Saved', path: '/skills/structures/new-structure/manifest.yaml' }),
      })

      const result = await saveStructure('new-structure', manifest)

      expect(mockFetch).toHaveBeenCalledWith('/api/structures/new-structure', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(manifest),
      })
      expect(result.path).toContain('new-structure')
    })
  })
})

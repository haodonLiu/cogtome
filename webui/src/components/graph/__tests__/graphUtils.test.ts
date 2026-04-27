import { describe, it, expect } from 'vitest'
import {
  graphToJson,
  jsonToGraph,
  autoLayout,
  createStartNode,
  createUnitNode,
  createEdge,
} from '../graphUtils'
import type { GraphNode, GraphEdge } from '../../../types'

describe('graphUtils', () => {
  describe('graphToJson', () => {
    it('produces correct MotifManifestV2 structure', () => {
      const nodes: GraphNode[] = [
        { id: 'start', type: 'start', position: { x: 0, y: 0 }, data: {} },
        { id: 'unit-1', type: 'unit', position: { x: 100, y: 0 }, data: { unit: 'test-unit' } },
      ]
      const edges: GraphEdge[] = [
        { id: 'e-start-unit-1', source: 'start', sourceHandle: 'output', target: 'unit-1', targetHandle: 'input' },
      ]

      const result = graphToJson(nodes, edges, 'test-motif')

      expect(result).toEqual({
        name: 'test-motif',
        type: 'motif',
        version: '2.0',
        graph: {
          nodes,
          edges,
        },
      })
    })

    it('handles empty nodes and edges', () => {
      const result = graphToJson([], [], 'empty-motif')

      expect(result).toEqual({
        name: 'empty-motif',
        type: 'motif',
        version: '2.0',
        graph: { nodes: [], edges: [] },
      })
    })
  })

  describe('jsonToGraph', () => {
    it('converts MotifManifestV2 to Graph format', () => {
      const manifest = {
        name: 'test',
        type: 'motif' as const,
        version: '2.0',
        graph: {
          nodes: [
            { id: 'start', type: 'start' as const, position: { x: 0, y: 0 }, data: {} },
          ],
          edges: [
            { id: 'e1', source: 'start', sourceHandle: 'output', target: 'end', targetHandle: 'input' },
          ],
        },
      }

      const result = jsonToGraph(manifest)

      expect(result).toEqual({
        nodes: manifest.graph.nodes,
        edges: manifest.graph.edges,
      })
    })

    it('parses JSON string input', () => {
      const jsonString = JSON.stringify({
        name: 'test',
        type: 'motif',
        version: '2.0',
        graph: {
          nodes: [{ id: 'n1', type: 'unit', position: { x: 0, y: 0 }, data: {} }],
          edges: [],
        },
      })

      const result = jsonToGraph(jsonString)

      expect(result.nodes).toHaveLength(1)
      expect(result.nodes[0].id).toBe('n1')
    })

    it('returns empty graph when manifest has no graph', () => {
      const result = jsonToGraph({ name: 'test', type: 'motif', version: '2.0' } as any)

      expect(result).toEqual({ nodes: [], edges: [] })
    })
  })

  describe('autoLayout', () => {
    it('returns nodes with positions in grid pattern', () => {
      const nodes: GraphNode[] = [
        { id: 'n1', type: 'unit', position: { x: 0, y: 0 }, data: {} },
        { id: 'n2', type: 'unit', position: { x: 0, y: 0 }, data: {} },
        { id: 'n3', type: 'unit', position: { x: 0, y: 0 }, data: {} },
      ]

      const result = autoLayout(nodes)

      expect(result).toHaveLength(3)
      expect(result[0].position).toEqual({ x: 50, y: 50 })
      expect(result[1].position).toEqual({ x: 300, y: 50 })
      expect(result[2].position).toEqual({ x: 550, y: 50 })
    })

    it('places nodes in multiple rows when > 5 nodes', () => {
      const nodes = Array.from({ length: 7 }, (_, i) => ({
        id: `n${i}`,
        type: 'unit' as const,
        position: { x: 0, y: i * 100 },
        data: {},
      }))

      const result = autoLayout(nodes)

      // Row 1: nodes 0-4, Row 2: nodes 5-6
      expect(result[0].position).toEqual({ x: 50, y: 50 })
      expect(result[4].position).toEqual({ x: 1050, y: 50 })
      expect(result[5].position).toEqual({ x: 50, y: 200 })
      expect(result[6].position).toEqual({ x: 300, y: 200 })
    })

    it('does not mutate original nodes', () => {
      const nodes: GraphNode[] = [
        { id: 'n1', type: 'unit', position: { x: 0, y: 0 }, data: {} },
      ]
      const originalPosition = { ...nodes[0].position }

      autoLayout(nodes)

      expect(nodes[0].position).toEqual(originalPosition)
    })
  })

  describe('roundtrip', () => {
    it('graphToJson and jsonToGraph preserve data', () => {
      const originalNodes: GraphNode[] = [
        createStartNode(0, 0),
        createUnitNode('unit-1', 'my-unit', 100, 0),
        createUnitNode('unit-2', 'other-unit', 200, 0),
      ]
      const originalEdges: GraphEdge[] = [
        createEdge('start', 'unit-1'),
        createEdge('unit-1', 'unit-2'),
      ]

      const json = graphToJson(originalNodes, originalEdges, 'roundtrip-test')
      const restored = jsonToGraph(json)

      expect(restored.nodes).toEqual(originalNodes)
      expect(restored.edges).toEqual(originalEdges)
    })
  })
})

import { describe, it, expect } from 'vitest'
import type {
  GraphNode,
  GraphEdge,
  Graph,
  MotifManifestV2,
  Position,
  Port,
  NodeType,
} from '../../types'

describe('GraphNode Serialization', () => {
  it('serializes all NodeType variants', () => {
    const nodeTypes: NodeType[] = ['start', 'unit', 'if', 'match', 'foreach', 'fork', 'join', 'return', 'motif']

    const nodes: GraphNode[] = nodeTypes.map((type, i) => ({
      id: `node-${type}`,
      type,
      position: { x: i * 100, y: i * 50 },
      data: {
        name: type === 'unit' ? 'test-unit' : undefined,
        condition: type === 'if' ? 'x > 0' : undefined,
        over: type === 'foreach' ? '${items}' : undefined,
        motif: type === 'motif' ? 'nested-motif' : undefined,
      },
    }))

    // Should serialize without errors
    const json = JSON.stringify(nodes)
    const parsed = JSON.parse(json) as GraphNode[]

    expect(parsed).toHaveLength(nodeTypes.length)
    expect(parsed.map(n => n.type)).toEqual(nodeTypes)
  })

  it('preserves Position with x/y coordinates', () => {
    const position: Position = { x: 123.45, y: -67.89 }
    const node: GraphNode = {
      id: 'test',
      type: 'unit',
      position,
      data: {},
    }

    const json = JSON.stringify(node)
    const parsed = JSON.parse(json)

    expect(parsed.position).toEqual({ x: 123.45, y: -67.89 })
  })

  it('handles Port arrays in outputs', () => {
    const ports: Port[] = [
      { id: 'out-1', name: 'result', type: 'string' },
      { id: 'out-2', name: 'error', type: 'boolean' },
    ]

    const node: GraphNode = {
      id: 'test',
      type: 'unit',
      position: { x: 0, y: 0 },
      data: { outputs: ports },
    }

    const json = JSON.stringify(node)
    const parsed = JSON.parse(json)

    expect(parsed.data.outputs).toHaveLength(2)
    expect(parsed.data.outputs[0].type).toBe('string')
  })

  it('handles nested subgraph in motif nodes', () => {
    const subgraph: Graph = {
      nodes: [
        { id: 'sub-start', type: 'start', position: { x: 0, y: 0 }, data: {} },
      ],
      edges: [],
    }

    const node: GraphNode = {
      id: 'motif-node',
      type: 'motif',
      position: { x: 0, y: 0 },
      data: { subgraph },
    }

    const json = JSON.stringify(node)
    const parsed = JSON.parse(json)

    expect(parsed.data.subgraph.nodes).toHaveLength(1)
  })
})

describe('GraphEdge Serialization', () => {
  it('serializes edge with all required fields', () => {
    const edge: GraphEdge = {
      id: 'e-start-unit',
      source: 'start',
      sourceHandle: 'default',
      target: 'unit-1',
      targetHandle: 'default',
      label: 'flow',
    }

    const json = JSON.stringify(edge)
    const parsed = JSON.parse(json) as GraphEdge

    expect(parsed.id).toBe('e-start-unit')
    expect(parsed.source).toBe('start')
    expect(parsed.target).toBe('unit-1')
    expect(parsed.label).toBe('flow')
  })

  it('handles edges without optional label', () => {
    const edge: GraphEdge = {
      id: 'e1',
      source: 'n1',
      sourceHandle: 'default',
      target: 'n2',
      targetHandle: 'default',
    }

    const json = JSON.stringify(edge)
    const parsed = JSON.parse(json) as GraphEdge

    expect(parsed.label).toBeUndefined()
  })

  it('preserves handle IDs for multi-output nodes', () => {
    const edges: GraphEdge[] = [
      { id: 'e-true', source: 'if-1', sourceHandle: 'true', target: 'unit-1', targetHandle: 'default' },
      { id: 'e-false', source: 'if-1', sourceHandle: 'false', target: 'unit-2', targetHandle: 'default' },
    ]

    const json = JSON.stringify(edges)
    const parsed = JSON.parse(json) as GraphEdge[]

    expect(parsed[0].sourceHandle).toBe('true')
    expect(parsed[1].sourceHandle).toBe('false')
  })
})

describe('Graph Container Serialization', () => {
  it('serializes complete graph with nodes and edges', () => {
    const graph: Graph = {
      nodes: [
        { id: 'start', type: 'start', position: { x: 0, y: 0 }, data: {} },
        { id: 'unit-1', type: 'unit', position: { x: 100, y: 0 }, data: { unit: 'echo' } },
        { id: 'if-1', type: 'if', position: { x: 200, y: 0 }, data: { condition: 'x > 0' } },
        { id: 'return-1', type: 'return', position: { x: 300, y: 0 }, data: {} },
      ],
      edges: [
        { id: 'e1', source: 'start', target: 'unit-1', sourceHandle: 'default', targetHandle: 'default' },
        { id: 'e2', source: 'unit-1', target: 'if-1', sourceHandle: 'default', targetHandle: 'default' },
        { id: 'e3', source: 'if-1', target: 'return-1', sourceHandle: 'default', targetHandle: 'default' },
      ],
    }

    const json = JSON.stringify(graph)
    const parsed = JSON.parse(json) as Graph

    expect(parsed.nodes).toHaveLength(4)
    expect(parsed.edges).toHaveLength(3)
  })
})

describe('MotifManifestV2 Serialization', () => {
  it('serializes complete motif manifest for API', () => {
    const manifest: MotifManifestV2 = {
      name: 'test-motif',
      type: 'motif',
      version: '2.0',
      description: 'Test motif',
      required_units: ['echo', 'transform'],
      graph: {
        nodes: [
          { id: 'start', type: 'start', position: { x: 0, y: 0 }, data: {} },
        ],
        edges: [],
      },
      input_schema: {
        type: 'object',
        properties: { x: { type: 'string' } },
        required: ['x'],
      },
      output_schema: {
        type: 'object',
        properties: { result: { type: 'string' } },
      },
    }

    const json = JSON.stringify(manifest)
    const parsed = JSON.parse(json) as MotifManifestV2

    expect(parsed.name).toBe('test-motif')
    expect(parsed.type).toBe('motif')
    expect(parsed.version).toBe('2.0')
    expect(parsed.required_units).toEqual(['echo', 'transform'])
    expect(parsed.input_schema?.required).toContain('x')
    expect(parsed.graph.nodes).toHaveLength(1)
  })

  it('roundtrips through JSON without data loss', () => {
    const original: MotifManifestV2 = {
      name: 'complex-motif',
      type: 'motif',
      version: '2.0',
      graph: {
        nodes: [
          { id: 'start', type: 'start', position: { x: 0, y: 0 }, data: {} },
          { id: 'foreach-1', type: 'foreach', position: { x: 100, y: 0 }, data: { over: '${items}', as_var: 'item', maxIterations: 50, parallel: true } },
          { id: 'unit-1', type: 'unit', position: { x: 200, y: 0 }, data: { unit: 'process', inputs: { item: '${item}' } } },
          { id: 'return-1', type: 'return', position: { x: 300, y: 0 }, data: {} },
        ],
        edges: [
          { id: 'e1', source: 'start', target: 'foreach-1', sourceHandle: 'default', targetHandle: 'default' },
          { id: 'e2', source: 'foreach-1', target: 'unit-1', sourceHandle: 'default', targetHandle: 'default' },
          { id: 'e3', source: 'unit-1', target: 'return-1', sourceHandle: 'default', targetHandle: 'default' },
        ],
      },
    }

    const json = JSON.stringify(original)
    const parsed = JSON.parse(json) as MotifManifestV2

    // Verify all data is preserved
    expect(parsed.name).toBe(original.name)
    expect(parsed.graph.nodes[1].data.over).toBe('${items}')
    expect(parsed.graph.nodes[1].data.maxIterations).toBe(50)
    expect(parsed.graph.nodes[1].data.parallel).toBe(true)
    expect(parsed.graph.edges).toHaveLength(3)
  })

  it('handles optional fields as undefined', () => {
    const minimal: MotifManifestV2 = {
      name: 'minimal',
      type: 'motif',
      graph: { nodes: [], edges: [] },
    }

    const json = JSON.stringify(minimal)
    const parsed = JSON.parse(json)

    expect(parsed.version).toBeUndefined()
    expect(parsed.description).toBeUndefined()
    expect(parsed.required_units).toBeUndefined()
  })
})

describe('Frontend ↔ Backend JSON Parity', () => {
  it('matches expected JSON structure for MotifManifestV2', () => {
    // This test verifies the JSON structure matches what the backend expects
    const manifest: MotifManifestV2 = {
      name: 'parity-test',
      type: 'motif',
      version: '2.0',
      graph: {
        nodes: [
          {
            id: 'n1',
            type: 'start',
            position: { x: 0, y: 0 },
            data: {},
          },
        ],
        edges: [
          {
            id: 'e1',
            source: 'n1',
            sourceHandle: 'default',
            target: 'n2',
            targetHandle: 'default',
          },
        ],
      },
    }

    const json = JSON.stringify(manifest, null, 2)

    // Verify structure keys match expected format
    expect(json).toContain('"name": "parity-test"')
    expect(json).toContain('"type": "motif"')
    expect(json).toContain('"version": "2.0"')
    expect(json).toContain('"graph":')
    expect(json).toContain('"nodes":')
    expect(json).toContain('"edges":')
  })

  it('supports all node types used by backend', () => {
    const allTypes: NodeType[] = ['start', 'unit', 'if', 'match', 'foreach', 'fork', 'join', 'return', 'motif']

    allTypes.forEach(type => {
      const node: GraphNode = {
        id: `test-${type}`,
        type,
        position: { x: 0, y: 0 },
        data: {},
      }

      const json = JSON.stringify(node)
      const parsed = JSON.parse(json)

      expect(parsed.type).toBe(type)
    })
  })
})

import { BlockNode, BlockEdge, Graph } from '../../types';
import YAML from 'yaml';

/**
 * COGTOME Motif YAML format:
 * name: <motif-name>
 * type: motif
 * flow:
 *   - name: step1
 *     unit: unit-name
 *     input:
 *       key: ${params.x}
 *   - name: step2
 *     foreach:
 *       over: ${items}
 *       max_iterations: 50
 *       parallel: false
 *       flow: [...]
 */

export function graphToYaml(nodes: BlockNode[], edges: BlockEdge[], name: string): string {
  const sorted = topologicalSort(nodes, edges);

  const flow = sorted.map((nodeId) => {
    const node = nodes.find((n) => n.id === nodeId);
    if (!node) return null;

    const incomingEdges = edges.filter((e) => e.target === nodeId);
    const inputMappings: Record<string, string> = {};

    incomingEdges.forEach((edge) => {
      const sourceNode = nodes.find((n) => n.id === edge.source);
      if (sourceNode && edge.targetHandle) {
        inputMappings[edge.targetHandle] = `\${steps.${sourceNode.data.name || edge.source}.output.${edge.sourceHandle}}`;
      }
    });

    if (node.data.inputs) {
      Object.entries(node.data.inputs).forEach(([key, val]) => {
        if (val && !Object.values(inputMappings).some((v) => v.includes(key))) {
          inputMappings[key] = val;
        }
      });
    }

    switch (node.type) {
      case 'unit':
        return {
          name: node.data.name || nodeId,
          unit: node.data.name,
          input: inputMappings,
        };
      case 'foreach':
        return {
          name: node.data.name || nodeId,
          foreach: {
            over: node.data.over || '',
            as_var: 'item',
            max_iterations: node.data.maxIterations || 50,
            parallel: node.data.parallel || false,
            flow: node.data.internalGraph
              ? graphToFlow(node.data.internalGraph.nodes, node.data.internalGraph.edges)
              : [],
          },
        };
      case 'if':
        return {
          name: node.data.name || nodeId,
          if: {
            condition: node.data.condition || '',
            then: [],
            else: [],
          },
        };
      case 'return':
        return {
          return: node.data.mappings || {},
        };
      default:
        return null;
    }
  }).filter(Boolean);

  const doc = {
    name,
    type: 'motif',
    flow: flow.filter(Boolean),
  };

  return YAML.stringify(doc, { indent: 2 });
}

export function yamlToGraph(yamlString: string, name: string): Graph {
  const doc = YAML.parse(yamlString);
  if (!doc || !doc.flow) return { nodes: [], edges: [] };

  const nodes: BlockNode[] = [];
  const edges: BlockEdge[] = [];
  let xOffset = 50;

  doc.flow.forEach((step: any, index: number) => {
    const nodeId = `${step.name || step.unit || 'step'}-${index}`;
    const position = { x: xOffset, y: 150 };

    if (step.return) {
      nodes.push({
        id: nodeId,
        type: 'return',
        position,
        data: { mappings: step.return },
      });
    } else if (step.foreach) {
      const innerGraph = step.foreach.flow
        ? yamlToGraph(YAML.stringify({ flow: step.foreach.flow }), step.name || 'nested')
        : undefined;
      nodes.push({
        id: nodeId,
        type: 'foreach',
        position,
        data: {
          name: step.name,
          over: step.foreach.over,
          maxIterations: step.foreach.max_iterations,
          parallel: step.foreach.parallel,
          internalGraph: innerGraph,
        },
      });
    } else if (step.if) {
      nodes.push({
        id: nodeId,
        type: 'if',
        position,
        data: {
          name: step.name,
          condition: step.if.condition,
        },
      });
    } else if (step.unit) {
      nodes.push({
        id: nodeId,
        type: 'unit',
        position,
        data: {
          name: step.unit,
          inputs: step.input || {},
          outputs: [{ id: 'output', name: 'output', type: 'string' }],
        },
      });
    }

    xOffset += 220;
  });

  for (let i = 0; i < nodes.length - 1; i++) {
    edges.push({
      id: `edge-${i}`,
      source: nodes[i].id,
      sourceHandle: 'output',
      target: nodes[i + 1].id,
      targetHandle: 'input',
    });
  }

  return { nodes, edges };
}

function graphToFlow(nodes: BlockNode[], edges: BlockEdge[]): any[] {
  return nodes.map((node) => {
    const incomingEdges = edges.filter((e) => e.target === node.id);
    const inputMappings: Record<string, string> = {};

    incomingEdges.forEach((edge) => {
      const sourceNode = nodes.find((n) => n.id === edge.source);
      if (sourceNode && edge.targetHandle) {
        inputMappings[edge.targetHandle] = `\${steps.${sourceNode.data.name || edge.source}.output.${edge.sourceHandle}}`;
      }
    });

    switch (node.type) {
      case 'unit':
        return { name: node.data.name || node.id, unit: node.data.name, input: inputMappings };
      case 'return':
        return { return: node.data.mappings || {} };
      default:
        return null;
    }
  }).filter(Boolean);
}

function topologicalSort(nodes: BlockNode[], edges: BlockEdge[]): string[] {
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

export function autoLayout(nodes: BlockNode[]): BlockNode[] {
  const sorted = [...nodes].sort((a, b) => {
    if (Math.abs(a.position.y - b.position.y) > 50) {
      return a.position.y - b.position.y;
    }
    return a.position.x - b.position.x;
  });

  return sorted.map((node, i) => ({
    ...node,
    position: { x: 50 + (i % 5) * 250, y: 50 + Math.floor(i / 5) * 150 },
  }));
}
import React, { useCallback, useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  NodeChange,
  EdgeChange,
  Connection,
  useNodesState,
  useEdgesState,
  OnNodesChange,
  OnEdgesChange,
  OnConnect,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { UnitNode } from '../graph/nodes/UnitNode';
import { IfNode } from '../graph/nodes/IfNode';
import { ForeachNode } from '../graph/nodes/ForeachNode';
import { ReturnNode } from '../graph/nodes/ReturnNode';
import { MotifNode } from '../graph/nodes/MotifNode';
import { BlockPalette } from '../graph/Palette';
import { PropertyPanel } from './PropertyPanel';
import { getMotif, saveMotif, listMotifs } from '../../api/client';
import { BlockNode, BlockEdge, BlockType, MotifInfo } from '../../types';
import YAML from 'yaml';

const nodeTypes = {
  unit: UnitNode,
  if: IfNode,
  foreach: ForeachNode,
  return: ReturnNode,
  motif: MotifNode,
};

export function MotifEditor() {
  const { name } = useParams<{ name: string }>();
  const navigate = useNavigate();
  const [motifs, setMotifs] = useState<MotifInfo[]>([]);
  const [selectedNode, setSelectedNode] = useState<BlockNode | null>(null);
  const [selectedEdge, setSelectedEdge] = useState<BlockEdge | null>(null);
  const [nodes, setNodes, onNodesChange] = useNodesState<BlockNode>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<BlockEdge>([]);

  useEffect(() => {
    listMotifs().then(setMotifs).catch(() => {});
    if (name) {
      getMotif(name).then((yaml) => {
        const { nodes: parsedNodes, edges: parsedEdges } = parseMotifYaml(yaml);
        setNodes(parsedNodes.map(toRFNode));
        setEdges(parsedEdges.map(toRFEdge));
      }).catch(() => {});
    }
  }, [name]);

  const onConnect: OnConnect = useCallback(
    (connection: Connection) => {
      if (!connection.source || !connection.target) return;
      const edge: BlockEdge = {
        id: `edge-${Date.now()}`,
        source: connection.source,
        sourceHandle: connection.sourceHandle || '',
        target: connection.target,
        targetHandle: connection.targetHandle || '',
      };
      setEdges((eds) => [...eds, toRFEdge(edge)]);
    },
    [setEdges]
  );

  const onNodeClick = useCallback((_: React.MouseEvent, node: any) => {
    const blockNode = nodes.find((n) => n.id === node.id);
    setSelectedNode(blockNode || null);
    setSelectedEdge(null);
  }, [nodes]);

  const onEdgeClick = useCallback((_: React.MouseEvent, edge: any) => {
    const blockEdge = edges.find((e) => e.id === edge.id);
    setSelectedEdge(blockEdge || null);
    setSelectedNode(null);
  }, [edges]);

  const onNodesChangeHandler: OnNodesChange = useCallback(
    (changes: NodeChange[]) => {
      onNodesChange(changes);
    },
    [onNodesChange]
  );

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      const type = e.dataTransfer.getData('application/reactflow') as BlockType;
      const blockName = e.dataTransfer.getData('application/blockname');
      if (!type) return;

      const reactFlowBounds = e.currentTarget.getBoundingClientRect();
      const position = {
        x: e.clientX - reactFlowBounds.left - 80,
        y: e.clientY - reactFlowBounds.top - 30,
      };

      const id = `${type}-${Date.now()}`;
      const newNode = {
        id,
        type,
        position,
        data: createNodeData(type, blockName),
      };
      setNodes((nds) => [...nds, newNode as any]);
    },
    [setNodes]
  );

  const updateNodeData = useCallback(
    (id: string, data: Partial<BlockNode['data']>) => {
      setNodes((nds) =>
        nds.map((n) => (n.id === id ? { ...n, data: { ...n.data, ...data } } : n))
      );
      setSelectedNode((prev) => (prev && prev.id === id ? { ...prev, data: { ...prev.data, ...data } } : prev));
    },
    [setNodes]
  );

  const deleteNode = useCallback(
    (id: string) => {
      setNodes((nds) => nds.filter((n) => n.id !== id));
      setEdges((eds) => eds.filter((e) => e.source !== id && e.target !== id));
      setSelectedNode(null);
    },
    [setNodes, setEdges]
  );

  const deleteEdge = useCallback(
    (id: string) => {
      setEdges((eds) => eds.filter((e) => e.id !== id));
      setSelectedEdge(null);
    },
    [setEdges]
  );

  const handleSave = async () => {
    if (!name) return;
    const yaml = graphToYaml(nodes as BlockNode[], edges as BlockEdge[], name);
    await saveMotif(name, yaml);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 16, padding: '12px 16px', background: '#0f0f1a', borderBottom: '1px solid #3b3b5c' }}>
        <button onClick={() => navigate(-1)} style={buttonStyle}>← Back</button>
        <h2 style={{ margin: 0, flex: 1, color: '#e2e8f0', fontFamily: 'monospace' }}>Motif: {name}</h2>
        <button onClick={handleSave} style={buttonStyle}>Save</button>
      </div>

      {/* Body */}
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        <BlockPalette
          motifs={motifs}
          onDragStart={(type, blockName) => {
            // Store drag data
          }}
        />

        <div
          style={{ flex: 1, position: 'relative' }}
          onDragOver={handleDragOver}
          onDrop={handleDrop}
        >
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChangeHandler}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            onNodeClick={onNodeClick}
            onEdgeClick={onEdgeClick}
            nodeTypes={nodeTypes}
            fitView
            snapToGrid
            snapGrid={[16, 16]}
          >
            <Background />
            <Controls />
            <MiniMap />
          </ReactFlow>
        </div>

        <PropertyPanel
          selectedNode={selectedNode as BlockNode | null}
          selectedEdge={selectedEdge as BlockEdge | null}
          onUpdateNode={updateNodeData}
          onDeleteNode={deleteNode}
          onDeleteEdge={deleteEdge}
        />
      </div>
    </div>
  );
}

function createNodeData(type: BlockType, name?: string): BlockNode['data'] {
  switch (type) {
    case 'unit':
      return { name: name || '', inputs: { input: '' }, outputs: [{ id: 'out-1', name: 'output', type: 'string' }] };
    case 'if':
      return { condition: '' };
    case 'foreach':
      return { over: '', maxIterations: 50, parallel: false };
    case 'return':
      return { mappings: {} };
    case 'motif':
      return { name: name || '' };
    default:
      return {};
  }
}

function toRFNode(node: BlockNode) {
  return {
    id: node.id,
    type: node.type,
    position: node.position,
    data: node.data,
  };
}

function toRFEdge(edge: BlockEdge) {
  return {
    id: edge.id,
    source: edge.source,
    target: edge.target,
    sourceHandle: edge.sourceHandle,
    targetHandle: edge.targetHandle,
    type: 'smoothstep',
    animated: true,
  };
}

function parseMotifYaml(yamlString: string): { nodes: BlockNode[]; edges: BlockEdge[] } {
  const doc = YAML.parse(yamlString);
  if (!doc || !doc.flow) return { nodes: [], edges: [] };

  const nodes: BlockNode[] = [];
  const edges: BlockEdge[] = [];
  let xOffset = 50;

  doc.flow.forEach((step: any, index: number) => {
    const nodeId = `${step.name || step.unit || 'step'}-${index}`;
    const position = { x: xOffset, y: 150 };

    if (step.return) {
      nodes.push({ id: nodeId, type: 'return', position, data: { mappings: step.return } });
    } else if (step.foreach) {
      nodes.push({ id: nodeId, type: 'foreach', position, data: { name: step.name, over: step.foreach.over, maxIterations: step.foreach.max_iterations, parallel: step.foreach.parallel } });
    } else if (step.if) {
      nodes.push({ id: nodeId, type: 'if', position, data: { name: step.name, condition: step.if.condition } });
    } else if (step.unit) {
      nodes.push({ id: nodeId, type: 'unit', position, data: { name: step.unit, inputs: step.input || {} } });
    }

    xOffset += 220;
  });

  // Sequential edges
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

function graphToYaml(nodes: BlockNode[], edges: BlockEdge[], name: string): string {
  const flow = nodes.map((node) => {
    const incomingEdges = edges.filter((e) => e.target === node.id);
    const inputMappings: Record<string, string> = {};

    incomingEdges.forEach((edge) => {
      const sourceNode = nodes.find((n) => n.id === edge.source);
      if (sourceNode && edge.targetHandle) {
        inputMappings[edge.targetHandle] = `\${steps.${sourceNode.data.name || edge.source}.output.${edge.sourceHandle}}`;
      }
    });

    if (node.data.inputs) {
      Object.entries(node.data.inputs).forEach(([key, val]) => {
        if (val) inputMappings[key] = val;
      });
    }

    switch (node.type) {
      case 'unit':
        return { name: node.data.name || node.id, unit: node.data.name, input: inputMappings };
      case 'foreach':
        return { name: node.data.name || node.id, foreach: { over: node.data.over || '', as_var: 'item', max_iterations: node.data.maxIterations || 50, parallel: node.data.parallel || false, flow: [] } };
      case 'if':
        return { name: node.data.name || node.id, if: { condition: node.data.condition || '', then: [], else: [] } };
      case 'return':
        return { return: node.data.mappings || {} };
      default:
        return null;
    }
  }).filter(Boolean);

  return YAML.stringify({ name, type: 'motif', flow }, { indent: 2 });
}

const buttonStyle: React.CSSProperties = {
  background: '#7c3aed',
  color: '#fff',
  border: 'none',
  padding: '8px 16px',
  borderRadius: 4,
  cursor: 'pointer',
  fontFamily: 'monospace',
};
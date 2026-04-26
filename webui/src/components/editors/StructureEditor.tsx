import React, { useCallback, useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { nodeTypes } from '../graph/nodes';
import { BlockPalette } from '../graph/Palette';
import { PropertyPanel } from './PropertyPanel';
import { getStructure, saveStructure, listMotifs } from '../../api/client';
import { BlockNode, BlockEdge, BlockType, MotifInfo } from '../../types/index';
import { autoLayout, yamlToGraph, graphToYaml } from '../graph/graphUtils';
import YAML from 'yaml';

export function StructureEditor() {
  const { name } = useParams<{ name: string }>();
  const navigate = useNavigate();
  const [motifs, setMotifs] = useState<MotifInfo[]>([]);
  const [selectedNode, setSelectedNode] = useState<BlockNode | null>(null);
  const [selectedEdge, setSelectedEdge] = useState<BlockEdge | null>(null);
  const [viewMode, setViewMode] = useState<'graph' | 'yaml'>('graph');
  const [nodes, setNodes, onNodesChange] = useNodesState<BlockNode>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<BlockEdge>([]);

  useEffect(() => {
    listMotifs().then(setMotifs).catch(() => {});
    if (name) {
      getStructure(name).then((manifest) => {
        const { nodes: parsedNodes, edges: parsedEdges } = parseStructureManifest(manifest, motifs);
        setNodes(parsedNodes.map(toRFNode));
        setEdges(parsedEdges.map(toRFEdge));
      }).catch(() => {});
    }
  }, [name]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Delete' || e.key === 'Backspace') {
        if (selectedNode) deleteNode(selectedNode.id);
        if (selectedEdge) deleteEdge(selectedEdge.id);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [selectedNode, selectedEdge]);

  const onConnect = useCallback(
    (connection: any) => {
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
    const manifest = graphToStructureManifest(nodes as BlockNode[], edges as BlockEdge[], name);
    await saveStructure(name, manifest);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 16, padding: '12px 16px', background: '#0f0f1a', borderBottom: '1px solid #3b3b5c' }}>
        <button onClick={() => navigate(-1)} style={buttonStyle}>← Back</button>
        <h2 style={{ margin: 0, color: '#e2e8f0', fontFamily: 'monospace' }}>Structure: {name}</h2>
        <div style={{ display: 'flex', gap: 4, flex: 1 }} />
        <div style={{ display: 'flex', gap: 4 }}>
          <button
            onClick={() => setViewMode('graph')}
            style={{ ...viewButtonStyle, background: viewMode === 'graph' ? '#7c3aed' : '#1a1a2e' }}
          >
            Graph
          </button>
          <button
            onClick={() => setViewMode('yaml')}
            style={{ ...viewButtonStyle, background: viewMode === 'yaml' ? '#7c3aed' : '#1a1a2e' }}
          >
            YAML
          </button>
        </div>
        <button onClick={handleSave} style={buttonStyle}>Save</button>
        <button
          onClick={() => {
            const layouted = autoLayout(nodes as BlockNode[]);
            setNodes(layouted.map(toRFNode) as any);
          }}
          style={{ ...buttonStyle, background: '#1a1a2e' }}
        >
          Auto Layout
        </button>
      </div>

      {/* Body */}
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        <BlockPalette
          motifs={motifs}
          onDragStart={(type, blockName) => {}}
        />

        {viewMode === 'yaml' ? (
          <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
            <textarea
              value={graphToYaml(nodes as BlockNode[], edges as BlockEdge[], name || '')}
              onChange={(e) => {
                try {
                  const { nodes: parsed } = yamlToGraph(e.target.value, name || '');
                  setNodes(parsed.map(toRFNode) as any);
                  setEdges([]);
                } catch {}
              }}
              style={{
                flex: 1,
                background: '#0f0f1a',
                color: '#e2e8f0',
                fontFamily: 'monospace',
                fontSize: 13,
                padding: 16,
                border: 'none',
                resize: 'none',
              }}
            />
          </div>
        ) : (
          <div
            style={{ flex: 1, position: 'relative' }}
            onDragOver={handleDragOver}
            onDrop={handleDrop}
          >
            <ReactFlow
              nodes={nodes}
              edges={edges}
              onNodesChange={onNodesChange}
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
        )}

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
    case 'motif':
      return { name: name || '', expanded: false };
    case 'if':
      return { condition: '' };
    case 'foreach':
      return { over: '', maxIterations: 50, parallel: false };
    case 'return':
      return { mappings: {} };
    default:
      return {};
  }
}

function toRFNode(node: BlockNode) {
  return { id: node.id, type: node.type, position: node.position, data: node.data };
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

function parseStructureManifest(manifest: any, motifs: MotifInfo[]): { nodes: BlockNode[]; edges: BlockEdge[] } {
  const nodes: BlockNode[] = [];
  const edges: BlockEdge[] = [];
  let xOffset = 50;

  if (!manifest || !manifest.motifs) return { nodes, edges };

  manifest.motifs.forEach((ref: any) => {
    const nodeId = `${ref.name}-${xOffset}`;
    nodes.push({
      id: nodeId,
      type: 'motif',
      position: { x: xOffset, y: 150 },
      data: { name: ref.name, expanded: false },
    });
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

function graphToStructureManifest(nodes: BlockNode[], edges: BlockEdge[], name: string): any {
  const motifRefs = nodes
    .filter((n) => n.type === 'motif')
    .map((n) => ({ name: n.data.name || '' }));

  return {
    name,
    type: 'structure',
    motifs: motifRefs,
    input_schema: undefined,
    output_schema: undefined,
  };
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

const viewButtonStyle: React.CSSProperties = {
  background: '#1a1a2e',
  color: '#e2e8f0',
  border: '1px solid #3b3b5c',
  padding: '6px 12px',
  borderRadius: 4,
  cursor: 'pointer',
  fontSize: 12,
  fontFamily: 'monospace',
};

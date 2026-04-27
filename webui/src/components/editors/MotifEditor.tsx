import { useCallback, useEffect, useState, useMemo, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  Connection,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { nodeTypes } from '../graph/nodes';
import { BlockPalette } from '../graph/Palette';
import { PropertyPanel } from './PropertyPanel';
import { getMotif, saveMotif, listMotifs } from '../../api/client';
import { BlockType, MotifInfo, GraphNode, GraphEdge } from '../../types';
import { autoLayout, jsonToGraph, graphToJson } from '../graph/graphUtils';

export function MotifEditor() {
  const { name } = useParams<{ name: string }>();
  const navigate = useNavigate();
  const [motifs, setMotifs] = useState<MotifInfo[]>([]);
  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);
  const [selectedEdge, setSelectedEdge] = useState<GraphEdge | null>(null);
  const [viewMode, setViewMode] = useState<'graph' | 'json'>('graph');
  const [nodes, setNodes] = useState<GraphNode[]>([]);
  const [edges, setEdges] = useState<GraphEdge[]>([]);
  const [collapsePalette, setCollapsePalette] = useState(false);
  const nodesRef = useRef(nodes);
  const edgesRef = useRef(edges);

  // Keep refs in sync
  useEffect(() => { nodesRef.current = nodes; }, [nodes]);
  useEffect(() => { edgesRef.current = edges; }, [edges]);

  // Memoized JSON for textarea
  const jsonContent = useMemo(() =>
    JSON.stringify(graphToJson(nodes, edges, name || ''), null, 2),
    [nodes, edges, name]
  );

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

  useEffect(() => {
    listMotifs().then(setMotifs).catch(console.error);
    if (name) {
      getMotif(name).then((manifest) => {
        const { nodes: parsedNodes, edges: parsedEdges } = jsonToGraph(manifest);
        setNodes(parsedNodes);
        setEdges(parsedEdges);
      }).catch(console.error);
    }
  }, [name]);

  const onConnect = useCallback(
    (connection: Connection) => {
      if (!connection.source || !connection.target) return;
      const edge: GraphEdge = {
        id: `edge-${Date.now()}`,
        source: connection.source,
        sourceHandle: connection.sourceHandle || 'output',
        target: connection.target,
        targetHandle: connection.targetHandle || 'input',
      };
      setEdges((eds) => [...eds, edge]);
    },
    [setEdges]
  );

  const onNodeClick = useCallback((_: React.MouseEvent, node: { id: string }) => {
    const graphNode = nodes.find((n: GraphNode) => n.id === node.id);
    setSelectedNode(graphNode || null);
    setSelectedEdge(null);
  }, [nodes]);

  const onEdgeClick = useCallback((_: React.MouseEvent, edge: { id: string }) => {
    const graphEdge = edges.find((e: GraphEdge) => e.id === edge.id);
    setSelectedEdge(graphEdge || null);
    setSelectedNode(null);
  }, [edges]);

  const onNodesChange = useCallback((changes: any[]) => {
    setNodes((nds) => changes.reduce<GraphNode[]>((acc, change) => {
      if (change.type === 'position' && change.position) {
        return acc.map((n) => n.id === change.id ? { ...n, position: change.position } : n);
      }
      if (change.type === 'remove') {
        return acc.filter((n) => n.id !== change.id);
      }
      return acc;
    }, nds));
  }, []);

  const onEdgesChange = useCallback((changes: any[]) => {
    setEdges((eds) => changes.reduce<GraphEdge[]>((acc, change) => {
      if (change.type === 'remove') {
        return acc.filter((e) => e.id !== change.id);
      }
      return acc;
    }, eds));
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      const type = e.dataTransfer.getData('application/reactflow') as BlockType;
      if (!type) return;

      const reactFlowBounds = e.currentTarget.getBoundingClientRect();
      const position = {
        x: e.clientX - reactFlowBounds.left - 80,
        y: e.clientY - reactFlowBounds.top - 30,
      };

      const id = `${type}-${Date.now()}`;
      const newNode: GraphNode = {
        id,
        type,
        position,
        data: createNodeData(type),
      };
      setNodes((nds) => [...nds, newNode]);
    },
    [setNodes]
  );

  const updateNodeData = useCallback(
    (id: string, data: Partial<GraphNode['data']>) => {
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
    const manifest = graphToJson(nodes, edges, name);
    await saveMotif(name, manifest);
  };

  return (
    <div className="editor-shell">
      {/* Toolbar */}
      <div className="editor-toolbar">
        <button onClick={() => navigate(-1)} className="btn-secondary">← Back</button>
        <h2 className="editor-title">Motif: {name}</h2>
        <div className="editor-toolbar-spacer" />
        <div className="editor-toolbar-group">
          <button
            onClick={() => setViewMode('graph')}
            className={`editor-toolbar-btn ${viewMode === 'graph' ? 'active' : ''}`}
          >
            Graph
          </button>
          <button
            onClick={() => setViewMode('json')}
            className={`editor-toolbar-btn ${viewMode === 'json' ? 'active' : ''}`}
          >
            JSON
          </button>
        </div>
        <button onClick={handleSave} className="btn-primary">Save</button>
        <button
          onClick={() => {
            const layouted = autoLayout(nodes);
            setNodes(layouted);
          }}
          className="btn-secondary"
        >
          Auto Layout
        </button>
        <button
          onClick={() => setCollapsePalette(p => !p)}
          className="btn-ghost"
          title={collapsePalette ? 'Show Palette' : 'Hide Palette'}
        >
          {collapsePalette ? '▶' : '◀'}
        </button>
      </div>

      {/* Body */}
      <div className="editor-body">
        {!collapsePalette && (
          <BlockPalette
            motifs={motifs}
            onDragStart={() => {}}
          />
        )}

        {viewMode === 'json' ? (
          <div className="editor-canvas editor-canvas--json">
            <textarea
              value={jsonContent}
              onChange={(e) => {
                try {
                  const manifest = JSON.parse(e.target.value);
                  const { nodes: parsed } = jsonToGraph(manifest);
                  setNodes(parsed);
                  setEdges(parsed.length > 0 && 'edges' in (manifest as any) ? (manifest as any).graph.edges : []);
                } catch {}
              }}
              className="editor-textarea"
            />
          </div>
        ) : (
          <div
            className="editor-canvas"
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
          selectedNode={selectedNode}
          selectedEdge={selectedEdge}
          onUpdateNode={updateNodeData}
          onDeleteNode={deleteNode}
          onDeleteEdge={deleteEdge}
        />
      </div>
    </div>
  );
}

const createNodeData = (type: BlockType): GraphNode['data'] => {
  switch (type) {
    case 'unit':
      return { unit: '', inputs: {} };
    case 'if':
      return { condition: '' };
    case 'match':
      return { condition: '' };
    case 'foreach':
      return { over: '', as_var: 'item', maxIterations: 50, parallel: false, subgraph: { nodes: [], edges: [] } };
    case 'fork':
      return {};
    case 'join':
      return {};
    case 'return':
      return { values: {} };
    case 'motif':
      return { motif: '', inputs: {} };
    case 'start':
      return {};
    default:
      return {};
  }
};
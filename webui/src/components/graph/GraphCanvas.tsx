import React, { useCallback } from 'react';
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  Node,
  Edge,
  Connection,
  useNodesState,
  useEdgesState,
  NodeChange,
  EdgeChange,
  OnNodesChange,
  OnEdgesChange,
  OnConnect,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { BlockNode, BlockEdge } from '../../types';

// Convert BlockNode → ReactFlow Node
const toRFNode = (node: BlockNode): Node => ({
  id: node.id,
  type: node.type,
  position: node.position,
  data: node.data,
});

// Convert BlockEdge → ReactFlow Edge
const toRFEdge = (edge: BlockEdge): Edge => ({
  id: edge.id,
  source: edge.source,
  target: edge.target,
  sourceHandle: edge.sourceHandle,
  targetHandle: edge.targetHandle,
  type: 'smoothstep',
  animated: true,
});

interface GraphCanvasProps {
  onNodeClick?: (nodeId: string) => void;
  onEdgeClick?: (edgeId: string) => void;
  nodes: BlockNode[];
  edges: BlockEdge[];
  onNodesChange?: OnNodesChange;
  onEdgesChange?: OnEdgesChange;
  onConnect?: OnConnect;
  onNodeDragStop?: (nodeId: string, position: { x: number; y: number }) => void;
  nodeTypes?: Record<string, React.ComponentType<any>>;
}

export function GraphCanvas({
  onNodeClick,
  onEdgeClick,
  nodes,
  edges,
  onNodesChange,
  onEdgesChange,
  onConnect,
  onNodeDragStop,
  nodeTypes,
}: GraphCanvasProps) {
  const rfNodes = nodes.map(toRFNode);
  const rfEdges = edges.map(toRFEdge);

  return (
    <div style={{ width: '100%', height: '100%' }}>
      <ReactFlow
        nodes={rfNodes}
        edges={rfEdges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onNodeClick={(_, node) => onNodeClick?.(node.id)}
        onEdgeClick={(_, edge) => onEdgeClick?.(edge.id)}
        onNodeDragStop={(_, node) => onNodeDragStop?.(node.id, node.position)}
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
  );
}

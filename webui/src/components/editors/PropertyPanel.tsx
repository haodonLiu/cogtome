import React from 'react';
import { BlockNode, BlockEdge } from '../../types';

interface PropertyPanelProps {
  selectedNode: BlockNode | null;
  selectedEdge: BlockEdge | null;
  onUpdateNode: (id: string, data: Partial<BlockNode['data']>) => void;
  onDeleteNode: (id: string) => void;
  onDeleteEdge: (id: string) => void;
}

export function PropertyPanel({
  selectedNode,
  selectedEdge,
  onUpdateNode,
  onDeleteNode,
  onDeleteEdge,
}: PropertyPanelProps) {
  if (selectedEdge) {
    return (
      <div style={{ width: 280, background: '#0f0f1a', borderLeft: '1px solid #3b3b5c', padding: 16, fontFamily: 'monospace' }}>
        <h3 style={{ color: '#e2e8f0', marginTop: 0 }}>Edge</h3>
        <div style={{ color: '#64748b', fontSize: 12 }}>
          {selectedEdge.source}.{selectedEdge.sourceHandle} → {selectedEdge.target}.{selectedEdge.targetHandle}
        </div>
        <button
          onClick={() => onDeleteEdge(selectedEdge.id)}
          style={{ marginTop: 16, background: '#ef4444', color: '#fff', border: 'none', padding: '8px 16px', borderRadius: 4, cursor: 'pointer' }}
        >
          Delete Edge
        </button>
      </div>
    );
  }

  if (!selectedNode) {
    return (
      <div style={{ width: 280, background: '#0f0f1a', borderLeft: '1px solid #3b3b5c', padding: 16, fontFamily: 'monospace', color: '#64748b' }}>
        Select a node to edit properties
      </div>
    );
  }

  const { type, data, id } = selectedNode;

  return (
    <div style={{ width: 280, background: '#0f0f1a', borderLeft: '1px solid #3b3b5c', padding: 16, fontFamily: 'monospace', overflowY: 'auto' }}>
      <h3 style={{ color: '#e2e8f0', marginTop: 0, textTransform: 'capitalize' }}>{type}</h3>

      {(type === 'unit' || type === 'motif') && (
        <>
          <label style={labelStyle}>Name</label>
          <input
            value={data.name || ''}
            onChange={(e) => onUpdateNode(id, { name: e.target.value })}
            style={inputStyle}
            disabled={type === 'unit'}
          />
        </>
      )}

      {type === 'if' && (
        <>
          <label style={labelStyle}>Condition</label>
          <textarea
            value={data.condition || ''}
            onChange={(e) => onUpdateNode(id, { condition: e.target.value })}
            style={{ ...inputStyle, height: 60 }}
          />
        </>
      )}

      {type === 'foreach' && (
        <>
          <label style={labelStyle}>Over (expression)</label>
          <input
            value={data.over || ''}
            onChange={(e) => onUpdateNode(id, { over: e.target.value })}
            style={inputStyle}
          />
          <label style={labelStyle}>Max Iterations</label>
          <input
            type="number"
            value={data.maxIterations || 50}
            onChange={(e) => onUpdateNode(id, { maxIterations: Number(e.target.value) })}
            style={inputStyle}
          />
        </>
      )}

      {type === 'return' && data.mappings && (
        <>
          <label style={labelStyle}>Return Mappings</label>
          {Object.entries(data.mappings).map(([key, val]) => (
            <div key={key} style={{ marginBottom: 8 }}>
              <div style={{ color: '#64748b', fontSize: 11 }}>{key}</div>
              <input
                value={String(val)}
                onChange={(e) => onUpdateNode(id, { mappings: { ...data.mappings, [key]: e.target.value } })}
                style={inputStyle}
              />
            </div>
          ))}
        </>
      )}

      <button
        onClick={() => onDeleteNode(id)}
        style={{ marginTop: 16, background: '#ef4444', color: '#fff', border: 'none', padding: '8px 16px', borderRadius: 4, cursor: 'pointer' }}
      >
        Delete Node
      </button>
    </div>
  );
}

const labelStyle: React.CSSProperties = {
  display: 'block',
  color: '#94a3b8',
  fontSize: 12,
  marginBottom: 4,
  marginTop: 12,
};

const inputStyle: React.CSSProperties = {
  width: '100%',
  background: '#1a1a2e',
  border: '1px solid #3b3b5c',
  borderRadius: 4,
  padding: '6px 8px',
  color: '#e2e8f0',
  fontFamily: 'monospace',
  fontSize: 12,
  boxSizing: 'border-box',
};
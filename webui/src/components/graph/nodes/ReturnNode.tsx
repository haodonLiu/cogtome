import React, { memo } from 'react';
import { Handle, Position, NodeProps } from '@xyflow/react';

export const ReturnNode = memo(({ data, selected }: NodeProps) => {
  const { mappings = {} } = data;

  return (
    <div
      style={{
        background: '#1a2e1a',
        border: selected ? '2px solid #22c55e' : '2px solid #2d5a2d',
        borderRadius: 8,
        padding: 12,
        minWidth: 160,
        fontFamily: 'monospace',
      }}
    >
      <Handle
        type="target"
        position={Position.Left}
        style={{ background: '#22c55e', width: 10, height: 10, border: 'none' }}
      />

      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
        <div
          style={{
            width: 20,
            height: 20,
            borderRadius: 4,
            background: '#22c55e',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 10,
            fontWeight: 700,
            color: '#000',
          }}
        >
          R
        </div>
        <span style={{ color: '#e2e8f0', fontWeight: 600 }}>return</span>
      </div>

      {Object.entries(mappings).map(([key, val]) => (
        <div key={key} style={{ color: '#86efac', fontSize: 11, marginBottom: 2 }}>
          {key}: <span style={{ color: '#7dd3fc' }}>{String(val).slice(0, 20)}</span>
        </div>
      ))}

      {Object.keys(mappings).length === 0 && (
        <div style={{ color: '#64748b', fontSize: 11, fontStyle: 'italic' }}>(no mappings)</div>
      )}
    </div>
  );
});

ReturnNode.displayName = 'ReturnNode';

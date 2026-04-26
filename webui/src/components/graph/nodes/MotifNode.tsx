import React, { memo } from 'react';
import { Handle, Position, NodeProps } from '@xyflow/react';

export const MotifNode = memo(({ data, selected }: NodeProps) => {
  const { name = '', expanded = false } = data;

  return (
    <div
      style={{
        background: '#2e1a2e',
        border: selected ? '2px solid #a855f7' : '2px solid #5c3b6e',
        borderRadius: 8,
        padding: 12,
        minWidth: 160,
        fontFamily: 'monospace',
      }}
    >
      <Handle
        type="target"
        position={Position.Left}
        style={{ background: '#a855f7', width: 10, height: 10, border: 'none' }}
      />

      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
        <div
          style={{
            width: 20,
            height: 20,
            borderRadius: 4,
            background: '#a855f7',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 10,
            fontWeight: 700,
            color: '#fff',
          }}
        >
          M
        </div>
        <span style={{ color: '#e2e8f0', fontWeight: 600 }}>{name || 'motif'}</span>
      </div>

      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <span
          style={{
            color: expanded ? '#22c55e' : '#64748b',
            fontSize: 11,
          }}
        >
          {expanded ? '▼ expanded' : '▶ collapsed'}
        </span>
      </div>

      <Handle
        type="source"
        position={Position.Right}
        style={{ background: '#22c55e', width: 10, height: 10, border: 'none' }}
      />
    </div>
  );
});

MotifNode.displayName = 'MotifNode';

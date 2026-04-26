import React, { memo } from 'react';
import { Handle, Position, NodeProps } from '@xyflow/react';

export const UnitNode = memo(({ data, selected }: NodeProps) => {
  const { name, inputs = {}, outputs = [] } = data;

  return (
    <div
      style={{
        background: '#1e1e2e',
        border: selected ? '2px solid #7c3aed' : '2px solid #3b3b5c',
        borderRadius: 8,
        padding: 12,
        minWidth: 160,
        fontFamily: 'monospace',
        fontSize: 13,
      }}
    >
      {/* Input handle */}
      {Object.keys(inputs).length > 0 && (
        <Handle
          type="target"
          position={Position.Left}
          style={{
            background: '#7c3aed',
            width: 10,
            height: 10,
            border: 'none',
          }}
        />
      )}

      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
        <div
          style={{
            width: 20,
            height: 20,
            borderRadius: 4,
            background: '#7c3aed',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 10,
            fontWeight: 700,
            color: '#fff',
          }}
        >
          U
        </div>
        <span style={{ color: '#e2e8f0', fontWeight: 600 }}>{name || 'unit'}</span>
      </div>

      {/* Input fields preview */}
      <div style={{ marginBottom: 4 }}>
        {Object.entries(inputs).map(([key, val]) => (
          <div key={key} style={{ color: '#94a3b8', fontSize: 11, marginBottom: 2 }}>
            {key}: <span style={{ color: '#7dd3fc' }}>{String(val).slice(0, 20)}</span>
          </div>
        ))}
      </div>

      {/* Output handle */}
      {outputs.length > 0 && (
        <Handle
          type="source"
          position={Position.Right}
          style={{
            background: '#22c55e',
            width: 10,
            height: 10,
            border: 'none',
          }}
        />
      )}
    </div>
  );
});

UnitNode.displayName = 'UnitNode';
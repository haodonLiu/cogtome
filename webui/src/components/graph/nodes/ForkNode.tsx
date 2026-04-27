import { memo } from 'react';
import { Handle, Position, NodeProps } from '@xyflow/react';

export const ForkNode = memo(({ selected }: NodeProps) => {
  return (
    <div
      style={{
        background: '#1e1e2e',
        border: selected ? '3px solid #8b5cf6' : '2px solid #8b5cf680',
        borderRadius: 8,
        padding: 12,
        minWidth: 100,
        fontFamily: 'monospace',
        fontSize: 13,
        borderTopWidth: 4,
      }}
    >
      {/* Input handle */}
      <Handle
        type="target"
        position={Position.Left}
        style={{
          background: '#8b5cf6',
          width: 10,
          height: 10,
          border: 'none',
        }}
      />

      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
        <div
          style={{
            width: 20,
            height: 20,
            borderRadius: 4,
            background: '#8b5cf6',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 10,
            fontWeight: 700,
            color: '#fff',
          }}
        >
          ⇉
        </div>
        <span style={{ color: '#e2e8f0', fontWeight: 600 }}>fork</span>
      </div>

      <div style={{ color: '#64748b', fontSize: 10 }}>
        parallel branches
      </div>

      {/* Multiple output handles */}
      <Handle
        type="source"
        position={Position.Right}
        id="branch1"
        style={{
          background: '#8b5cf6',
          width: 8,
          height: 8,
          border: 'none',
          top: '25%',
        }}
      />
      <Handle
        type="source"
        position={Position.Right}
        id="branch2"
        style={{
          background: '#8b5cf6',
          width: 8,
          height: 8,
          border: 'none',
          top: '50%',
        }}
      />
      <Handle
        type="source"
        position={Position.Right}
        id="branch3"
        style={{
          background: '#8b5cf6',
          width: 8,
          height: 8,
          border: 'none',
          top: '75%',
        }}
      />
    </div>
  );
});

ForkNode.displayName = 'ForkNode';
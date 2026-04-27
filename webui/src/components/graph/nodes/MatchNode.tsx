import { memo } from 'react';
import { Handle, Position, NodeProps } from '@xyflow/react';

interface MatchNodeData {
  condition?: string;
}

export const MatchNode = memo(({ data, selected }: NodeProps) => {
  const nodeData = data as MatchNodeData;
  const { condition = '' } = nodeData;

  return (
    <div
      style={{
        background: '#1e1e2e',
        border: selected ? '2px solid #ec4899' : '2px solid #ec489980',
        borderRadius: 8,
        padding: '16px 20px',
        minWidth: 140,
        fontFamily: 'monospace',
        fontSize: 13,
        clipPath: 'polygon(25% 0%, 75% 0%, 100% 50%, 75% 100%, 25% 100%, 0% 50%)',
      }}
    >
      {/* Input handle */}
      <Handle
        type="target"
        position={Position.Left}
        style={{
          background: '#ec4899',
          width: 10,
          height: 10,
          border: 'none',
        }}
      />

      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
        <div
          style={{
            width: 20,
            height: 20,
            borderRadius: 4,
            background: '#ec4899',
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
        <span style={{ color: '#e2e8f0', fontWeight: 600 }}>match</span>
      </div>

      {/* Condition */}
      <div style={{ color: '#94a3b8', fontSize: 11 }}>
        on: <span style={{ color: '#ec4899' }}>{condition || '...'}</span>
      </div>

      {/* Multiple output handles */}
      <Handle
        type="source"
        position={Position.Right}
        id="case1"
        style={{
          background: '#ec4899',
          width: 8,
          height: 8,
          border: 'none',
          top: '30%',
        }}
      />
      <Handle
        type="source"
        position={Position.Right}
        id="case2"
        style={{
          background: '#ec4899',
          width: 8,
          height: 8,
          border: 'none',
          top: '70%',
        }}
      />
    </div>
  );
});

MatchNode.displayName = 'MatchNode';
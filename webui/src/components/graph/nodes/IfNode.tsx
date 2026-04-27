import { memo } from 'react';
import { Handle, Position, NodeProps } from '@xyflow/react';

interface IfNodeData {
  condition?: string;
  expanded?: boolean;
  body?: any[];
}

export const IfNode = memo(({ data, selected }: NodeProps) => {
  const nodeData = data as IfNodeData;
  const { condition = '', expanded = false } = nodeData;

  return (
    <div
      style={{
        background: '#1a1a2e',
        border: selected ? '2px solid #f59e0b' : '2px solid #3b3b5c',
        borderRadius: 8,
        padding: 12,
        minWidth: 180,
        fontFamily: 'monospace',
      }}
    >
      <Handle
        type="target"
        position={Position.Left}
        style={{ background: '#f59e0b', width: 10, height: 10, border: 'none' }}
      />

      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
        <div
          style={{
            width: 20,
            height: 20,
            borderRadius: 4,
            background: '#f59e0b',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 10,
            fontWeight: 700,
            color: '#000',
          }}
        >
          ?
        </div>
        <span style={{ color: '#e2e8f0', fontWeight: 600 }}>if</span>
      </div>

      <div style={{ background: '#0f0f1a', borderRadius: 4, padding: 8, marginBottom: 8 }}>
        <div style={{ color: '#64748b', fontSize: 10, marginBottom: 4 }}>condition</div>
        <div style={{ color: '#fbbf24', fontSize: 12 }}>{condition || '(none)'}</div>
      </div>

      {expanded && (
        <div style={{ color: '#64748b', fontSize: 11, fontStyle: 'italic' }}>
          (body: {nodeData.body?.length || 0} nodes)
        </div>
      )}

      <Handle
        type="source"
        position={Position.Right}
        style={{ background: '#22c55e', width: 10, height: 10, border: 'none' }}
      />
    </div>
  );
});

IfNode.displayName = 'IfNode';
